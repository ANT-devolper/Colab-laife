# Architecture

> **Status legend:** ✅ implemented · 🚧 planned. This page marks each piece so it never
> overstates the code.

## System overview

ColabLife is a multi-tenant SaaS. A Rust backend (Axum) exposes an HTTP/JSON API; an Elm
frontend consumes it. State is persisted in PostgreSQL through SeaORM.

```
Elm frontend  ──HTTP/JSON──>  Axum backend  ──SeaORM──>  PostgreSQL
```

✅ The backend skeleton (router + health probes), the cross-tenant schema, and password
hashing exist today. 🚧 The Elm frontend, the business modules and their endpoints are
planned.

## Backend workspace

The backend is a Cargo workspace (`backend/`) split into focused crates
(`backend/crates/*`):

| Crate | Responsibility | Status |
|---|---|---|
| `api` | Axum HTTP app: builds the router, wires shared state, serves the probes and `POST /organizations`. Entry point in `main`; routes in `build_router`. | ✅ probes, provisioning endpoint |
| `entity` | SeaORM entities (the persisted data model). | ✅ `organization`, `user` |
| `migration` | `sea-orm-migration`; defines `PublicMigrator` and `TenantMigrator`. Run via `cargo run -p migration` / `just migrate`. | ✅ public schema |
| `service` | Domain/business logic, kept independent of HTTP and (where possible) of the ORM. | ✅ password hashing, tenant provisioning |

The router is created by `build_router(db, database_url)` in
`backend/crates/api/src/lib.rs`, which is kept separate from `main` so integration tests can
drive the real routes over HTTP. Shared state (`AppState`) carries an
`Arc<DatabaseConnection>` plus the `database_url` (so handlers can open the search-path
connections that tenant provisioning needs).

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
  provisioning flow. Currently empty; tenant tables are appended as the domain model
  grows. 🚧

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

### Current data model (`public`)

- **`organization`** (`backend/crates/entity/src/organization.rs`) — `id` (UUID), unique
  `name`, `plan`, optional `due_date`, `employee_limit`, `is_active`, timestamps;
  `has_many` users.
- **`user`** (`backend/crates/entity/src/user.rs`) — `id` (UUID), `name`, unique `email`,
  `password_hash`, `is_admin`, `organization_id` (FK → organization), `deleted`
  (soft-delete), timestamps.

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
