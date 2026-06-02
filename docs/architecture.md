# Architecture

> **Status legend:** ✅ implemented · 🚧 planned. This page marks each piece so it never
> overstates the code.

## System overview

ColabLife is a multi-tenant SaaS. A Rust backend (Axum) exposes an HTTP/JSON API; an Elm
frontend consumes it. State is persisted in PostgreSQL through SeaORM.

```
Elm SPA  ──HTTP/JSON──>  Axum backend  ──SeaORM──>  PostgreSQL
   ▲                          │
   └──── served as static ────┘   (same origin, see ADR 0011)
```

✅ The backend skeleton (router + health probes), the cross-tenant schema, password hashing,
tenant provisioning, credential login (JWT), the authenticated request pipeline (per-request
tenant resolution + auth extractor), granular RBAC (per-tenant permissions + per-route guard)
and the `sector`/`role`/`collaborator` tenant-domain resources exist today — the foundation and
the cadastro backend are complete. The Elm frontend has its foundation: a login page that
obtains a session token, served from the Axum binary on the same origin. 🚧 Read-only lists in
the SPA and the remaining business modules with their endpoints are planned.

## Backend workspace

The backend is a Cargo workspace (`backend/`) split into focused crates
(`backend/crates/*`):

| Crate | Responsibility | Status |
|---|---|---|
| `api` | Axum HTTP app: builds the router, wires shared state, serves the probes, `POST /organizations`, `POST /auth/login`, `GET /auth/me`, the RBAC-guarded `GET /users` and the RBAC-guarded `/sectors`, `/roles`, `/collaborators`, `/feedbacks` and `/expectation-items` CRUD (via the `TenantContext` extractor). Entry point in `main`; routes in `build_router`. | ✅ probes, provisioning + auth endpoints, auth extractor + RBAC guard, sectors + roles + collaborators + feedback + expectation-contract CRUD |
| `entity` | SeaORM entities (the persisted data model). | ✅ `organization`, `user`, `permission::*`, `sector`, `role`, `collaborator`, `feedback`, `expectation_contract_item` |
| `migration` | `sea-orm-migration`; defines `PublicMigrator` and `TenantMigrator`. Run via `cargo run -p migration` / `just migrate`. | ✅ public schema, tenant RBAC + `sector` + `role` + `collaborator` + `feedback` + `expectation_contract_item` |
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
  provisioning flow. Currently: the RBAC tables (see Authorization) and the `sector`, `role`,
  `collaborator`, `feedback` and `expectation_contract_item` tables (see Tenant domain). ✅ More
  tenant tables are appended as the domain model grows. 🚧

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
- The per-route guard is `TenantContext::require(resource)`: admins (the `is_admin` claim)
  pass without a lookup; everyone else is checked against the chain via
  `service::permission::has_permission`, returning `403` when the resource is not granted. ✅
- `GET /users` (`backend/crates/api/src/users.rs`) is the first RBAC-guarded route: it
  requires `user.read` and lists the caller organization's users. Authorization is checked
  against the tenant schema; the listing reads identities from `public`. ✅

## Tenant domain (Cadastro)

The business domain tables live in the **tenant schema** (created by `TenantMigrator`), so
their handlers query the per-request connection from `TenantContext` (`ctx.tenant_db`) rather
than `public` (`state.db`). Every route runs `ctx.require(resource)` before any query, and
removal is a **soft delete** (`active = false`); listings filter to active rows.

- **`sector`** (`backend/crates/entity/src/sector.rs`, migration
  `m20260601_000004_create_sector`) — `id` (UUID), `name`, `active` (soft-delete flag),
  timestamps. Exposed by `api::sectors` as `GET`/`POST /sectors` and `PATCH`/`DELETE
  /sectors/{id}`, guarded by `sector.{read,create,update,delete}`. It is the first domain
  resource living in the tenant schema. ✅
- **`role`** (`backend/crates/entity/src/role.rs`, migration
  `m20260601_000005_create_role`) — `id` (UUID), `name`, the legacy description fields (all
  optional text: `profile_suggestion`, `objective`, the `requirement_*` breakdown and
  `observation`), `active` (soft-delete flag), timestamps. Exposed by `api::roles` as
  `GET`/`POST /roles` and `PATCH`/`DELETE /roles/{id}`, guarded by
  `role.{read,create,update,delete}`; `PATCH` updates only the fields present in the body. ✅
- **`collaborator`** (`backend/crates/entity/src/collaborator.rs`, migration
  `m20260601_000006_create_collaborator`) — the corporate record of a person: `id` (UUID),
  `name`, optional `sector_id`/`role_id` FKs, an optional `manager_id` self-FK (the hierarchy),
  `whatsapp`, `email`, `is_manager`, `date_of_hire`, `active` (soft-delete flag), timestamps.
  The optional `user_id` links to the `public.users` login **by value** (no cross-schema FK,
  the same approach as `permission_profile_users`). Exposed by `api::collaborators` as
  `GET`/`POST /collaborators` and `PATCH`/`DELETE /collaborators/{id}`, guarded by
  `collaborator.{read,create,update,delete}`; `create`/`update` reject a dangling
  sector/role/manager reference with `422`, and `PATCH` updates only the fields present in the
  body. The org-hierarchy/"accessible collaborators" service is 🚧 deferred (only the
  `manager_id` column exists for now). ✅
- **`feedback`** (`backend/crates/entity/src/feedback.rs`, migration
  `m20260601_000007_create_feedback`) — the first people domain (depends on `collaborator`): a
  structured feedback event with `collaborator_id` (FK), `feedback_date`, optional
  `next_feedback_date`, the expectation-contract observations (`…_observation` and
  `…_observation_private`), `status`, `active` (soft-delete flag) and timestamps. Manager/sector
  are **not** stored — they are derived from the collaborator at read time; AI/transcription is
  out of scope. Exposed by `api::feedback` as `GET`/`POST /feedbacks` and `PATCH`/`DELETE
  /feedbacks/{id}`, guarded by `feedback.{read,create,update,delete}`; `GET` lists newest-first
  with an optional `?collaborator_id=` filter, and `create` rejects an unknown collaborator with
  `422`. ✅
- **`expectation_contract_item`** (`backend/crates/entity/src/expectation_contract_item.rs`,
  migration `m20260601_000008_create_expectation_contract_item`) — a checklist entry of a
  feedback's expectation contract: `feedback_id` (FK), a `kind` discriminator
  (`goal`/`behavior`), `description`, `done`, `active`, timestamps. **Redesign:** the legacy
  model split this into two identical tables (`expectation_contract_goals` and
  `expectation_contract_behavior`); we unify them with `kind`. Exposed by
  `api::expectation_items` as `GET`/`POST /expectation-items` and `PATCH`/`DELETE
  /expectation-items/{id}`, guarded by `expectation.{read,create,update,delete}`; `GET` accepts
  optional `?feedback_id=`/`?kind=` filters, and `create` rejects an invalid `kind` or unknown
  feedback with `422`. `feedback_behavior` and annotations are 🚧 planned next. ✅

## Frontend & delivery

The frontend is an Elm SPA (`frontend/`, The Elm Architecture) that talks to the backend
over HTTP/JSON. Current scope:

- A sign-in page that exchanges credentials for a session token (`Api.elm` — the HTTP
  boundary; `Page/Login.elm` — the form; `Main.elm` — the `Login` / `Directory` shell). ✅
- A read-only **Directory** (`Page/Directory.elm`): once authenticated, it fetches
  `/collaborators`, `/sectors` and `/roles` with the Bearer token and renders a table per list,
  each with a loading / empty / error state. ✅
- Write actions from the UI (create/edit) are 🚧 planned for a later slice.

**Single-origin delivery** ([ADR 0011](adr/0011-serve-spa-from-axum.md)): the Axum binary
serves the built SPA itself, so the browser only ever talks to one origin (no CORS). ✅

- `api::with_static_spa(router, dist_dir)` adds a `tower-http` `ServeDir` as the router's
  fallback, with `index.html` as the fallback for unknown paths (client-side routing). API
  routes are matched first, so they keep precedence over static files.
- `main` enables it only when `FRONTEND_DIST` is set, so the same binary can run API-only
  (the default in pure-API tests). `just frontend-build` compiles the SPA into `frontend/dist`
  and `just run` points `FRONTEND_DIST` at it.
- Because static serving never touches the database, it is covered by a fast integration test
  using a `MockDatabase` (no Docker).

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
  (a throwaway PostgreSQL in Docker). Static SPA serving is integration-tested with a
  `MockDatabase` (no Docker).
- **Frontend unit** — `elm-test`. ✅ Covers the API boundary's pure parts (the login encoder
  and the response/list decoders).
- **E2E** — Playwright. ✅ A first spec drives the real stack: Playwright's `webServer`
  (`e2e/scripts/e2e-stack.mjs`) boots PostgreSQL, migrates, builds the SPA and runs the API
  serving it; the test provisions a tenant via the API, signs in through the SPA and asserts the
  empty directory. The chromium project is the gate (`npm test`); Firefox/WebKit are available
  via `npm run test:all` once their system libraries are installed.

## See also

- Decisions behind this design: [`adr/`](adr/).
- What each business module will do: [`domain/`](domain/).
