# Architecture

> **Status legend:** ✅ implemented · 🚧 planned. This page marks each piece so it never
> overstates the code.

## System overview

ColabLife is a multi-tenant SaaS. A Rust backend (Axum) exposes an HTTP/JSON API; an Elm
frontend consumes it. State is persisted in PostgreSQL through SeaORM.

```
Elm frontend  ──HTTP/JSON──>  Axum backend  ──SeaORM──>  PostgreSQL
```

✅ The backend skeleton (router + health probes), the cross-tenant schema, password hashing,
tenant provisioning, credential login (JWT) and the authenticated request pipeline (per-request
tenant resolution + auth extractor) exist today. 🚧 The Elm frontend, the RBAC permission guard,
and the business modules and their endpoints are planned.

## Backend workspace

The backend is a Cargo workspace (`backend/`) split into focused crates
(`backend/crates/*`):

| Crate | Responsibility | Status |
|---|---|---|
| `api` | Axum HTTP app: builds the router, wires shared state, serves the probes, `POST /organizations`, `POST /auth/login` and the authenticated `GET /auth/me` (via the `TenantContext` extractor). Entry point in `main`; routes in `build_router`. | ✅ probes, provisioning + auth endpoints, auth extractor |
| `entity` | SeaORM entities (the persisted data model). | ✅ `organization`, `user`, `permission::*` |
| `migration` | `sea-orm-migration`; defines `PublicMigrator` and `TenantMigrator`. Run via `cargo run -p migration` / `just migrate`. | ✅ public schema |
| `service` | Domain/business logic, kept independent of HTTP and (where possible) of the ORM. | ✅ password hashing, tenant provisioning, authentication, tenant registry |

The router is created by `build_router(db, database_url, jwt_secret)` in
`backend/crates/api/src/lib.rs`, which is kept separate from `main` so integration tests can
drive the real routes over HTTP. Shared state (`AppState`) carries an
`Arc<DatabaseConnection>`, the `database_url` (so handlers can open the search-path
connections that tenant provisioning needs) and the `jwt_secret` (read from `JWT_SECRET`,
used to sign and verify session tokens).

Business logic lives in `service` so it can be unit-tested without a database — e.g.
`hash_password` / `verify_password` over Argon2 in
`backend/crates/service/src/password.rs` (PHC strings, per-password random salt).

## Multi-tenancy (schema-per-tenant)

Tenancy is modeled as **one PostgreSQL schema per tenant**, with a shared `public` schema
for cross-tenant identity:

- **`public` schema** — global tables. Today: `organizations` (the tenant root) and
  `users` (login identities). `users.email` is globally unique so login can resolve the
  owning tenant; `organizations.name` doubles as the slug of the tenant's dedicated schema.
- **Per-tenant schema** — holds that tenant's domain tables, isolated from other tenants.

Migrations are split accordingly in `backend/crates/migration/src/lib.rs`:

- `PublicMigrator` — migrations for the `public` schema; run day-to-day via
  `cargo run -p migration`. Currently: create `organizations`, then `users`. ✅
- `TenantMigrator` — migrations applied inside each tenant's schema, run by the tenant
  provisioning flow. Currently: the RBAC tables (see Authorization). ✅ More tenant tables
  are appended as the domain model grows. 🚧

### Tenant provisioning

`service::provisioning::provision_organization` turns a request for a new tenant into a
migrated schema plus an admin user ([ADR 0007](adr/0007-tenant-provisioning.md)): ✅ (service
level; HTTP endpoint 🚧)

1. Validate the organization `name` as a safe SQL identifier (it doubles as the schema slug).
2. `CREATE SCHEMA`, then run `TenantMigrator` against a connection whose `search_path` points
   at it (SeaORM `ConnectOptions::set_schema_search_path`). This same option is how a tenant
   schema is targeted, including the planned per-request resolver.
3. Insert the organization and its Argon2-hashed admin user in one `public` transaction.

Cross-step atomicity is best-effort (schema DDL is non-transactional); a failed migration
drops the new schema. Hardening is deferred.

It is exposed over HTTP as `POST /organizations` (`backend/crates/api/src/organizations.rs`):
`201` with the created organization and admin on success; `400` for an invalid name, `409`
for a duplicate, `500` otherwise.

### Per-request tenant resolution

Once a tenant exists, each request reaches its schema through a `TenantRegistry`
(`backend/crates/service/src/tenant.rs`), [ADR 0009](adr/0009-per-request-tenant-resolution.md): ✅

- It caches one `DatabaseConnection` per schema (`RwLock<HashMap<schema, Arc<…>>>`), each
  opened with `ConnectOptions::set_schema_search_path` — the same mechanism provisioning uses.
- `connection(schema)` validates the name (shared `is_valid_schema_name`), returns the cached
  connection or opens and caches a new one (double-checked, so one connection per schema).
- It does not verify the schema exists; a missing schema surfaces as a later query error.

The auth extractor (`TenantContext`) reads the tenant from the request's token and hands
handlers this connection (see Authentication). ✅

### Current data model (`public`)

- **`organization`** (`backend/crates/entity/src/organization.rs`) — `id` (UUID), unique
  `name`, `plan`, optional `due_date`, `employee_limit`, `is_active`, timestamps;
  `has_many` users.
- **`user`** (`backend/crates/entity/src/user.rs`) — `id` (UUID), `name`, unique `email`,
  `password_hash`, `is_admin`, `organization_id` (FK → organization), `deleted`
  (soft-delete), timestamps.

## Authentication

Authentication is native and **stateless** ([ADR 0008](adr/0008-stateless-jwt-sessions.md)): ✅

- `POST /auth/login` (`backend/crates/api/src/auth.rs`) takes `{ email, password }`,
  verifies them against `public` and returns a signed **JWT** on success
  (`200 { token, token_type: "Bearer" }`). Wrong/unknown credentials → `401`; a deactivated
  organization → `403`.
- `service::auth::authenticate` does the verification: it looks up a non-deleted user by
  email and checks the Argon2 hash. Unknown email, soft-deleted user and wrong password all
  collapse to one `InvalidCredentials` outcome (no user enumeration); a valid user whose
  organization is inactive yields `OrganizationInactive`.
- The token carries `Claims { sub, org, schema, is_admin, exp }` (HS256, signed with
  `JWT_SECRET`, 24h expiry). `schema` lets the auth extractor resolve the tenant from the
  token alone. Encoding/decoding live in `service::auth` (`encode_token` / `decode_token`).
- Protected routes take the `TenantContext` extractor
  (`backend/crates/api/src/extract.rs`, `FromRequestParts`): it reads
  `Authorization: Bearer <jwt>`, validates it, and resolves the tenant connection from the
  `TenantRegistry`. Missing/malformed/invalid/expired token → `401`; an unreachable tenant
  connection → `500`. Routes that omit the extractor stay public. ✅
- `GET /auth/me` is the first protected route: it returns the caller's identity
  (`{ user_id, organization_id, schema, is_admin }`) from the verified token.
- A per-route **RBAC permission guard** building on `TenantContext` is 🚧 planned.

## Authorization (RBAC)

Authorization is granular and per-tenant ([ADR 0010](adr/0010-granular-rbac.md)). A user
holds **profiles**; a profile groups **tasks**; a task groups **resources** (the protected
actions). A caller's permissions are the resources reachable via
`profile_users → profiles → profile_tasks → tasks → task_resources → resources`.

- The six `permission_*` tables live in each **tenant schema** (created by `TenantMigrator`,
  `backend/crates/migration/src/tenant/`), with entities under
  `backend/crates/entity/src/permission/`. ✅
- Resources are identified as `domain.action` (e.g. `user.read`), not the legacy
  `res://…` URIs. `permission_profile_users.user_id` references `public.users` by value
  only (no cross-schema FK).
- Provisioning seeds a minimal resource catalog (`Resource::catalog()` in
  `service::permission` — user and RBAC management) plus an "administrator" profile that
  grants every resource, and links the tenant's admin user to it. ✅
- The per-route guard `TenantContext::require(resource)` (admin bypass via the token, `403`
  otherwise) is 🚧 planned.

## Health & operability

The `api` crate exposes two probes (`backend/crates/api/src/health.rs`): ✅

- `GET /health` — **liveness**. Returns `200 {"status":"ok"}` and deliberately does not
  touch the database, so it stays green while dependencies are down.
- `GET /health/ready` — **readiness**. Pings PostgreSQL; `200 {"status":"ready"}` on
  success, `503 {"status":"unavailable"}` when the database is unreachable.

## Testing strategy

The full suite must be green before every commit (see [`../CLAUDE.md`](../CLAUDE.md)).
Levels:

- **Backend unit** — `cargo test` with `rstest` and `mockall`; services are isolated from
  the ORM (e.g. the liveness probe runs against a SeaORM `MockDatabase`, no Docker).
- **Backend integration** — `axum-test` (real HTTP through the app) with `testcontainers`
  (a throwaway PostgreSQL in Docker).
- **Frontend unit** — `elm-test`.
- **E2E** — Playwright.

## See also

- Decisions behind this design: [`adr/`](adr/).
- What each business module will do: [`domain/`](domain/).
