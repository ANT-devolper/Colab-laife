# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Project documentation structure: `README.md` rewritten as an index (pitch, status,
  stack, minimal quickstart, links) and a `docs/` directory with `architecture.md`
  (system overview, crates, schema-per-tenant multi-tenancy, health probes, testing;
  planned-vs-implemented markers), `docs/domain/` (index, glossary and a page per business
  module from a common template), and `docs/adr/` (template plus ADRs 0001–0006 recording
  decisions already taken: ADRs themselves, Rust+Axum, SeaORM/PostgreSQL, multi-tenant
  schema-per-tenant, Elm, Argon2). `CONTRIBUTING.md` layout updated to list `docs/`.
- Mandatory documentation directive in `CLAUDE.md` ("Documentation — mandatory"): changes
  affecting architecture, API contracts or module behavior must update `docs/` in the same
  task before the commit, with a new ADR for meaningful architectural decisions; the commit
  flow now includes the docs step.
- Project guidelines and development process (XP pair programming, TDD with
  Red → Green → Refactor, no-regression rule, regression test for every bug, mandatory
  CHANGELOG, Conventional Commits) in `CLAUDE.md`.
- Commit policy: Claude commits automatically; pushing is manual (user only).
- Project stack definition: Rust + Axum + SeaORM over PostgreSQL (backend), Elm
  (frontend); deploy target and mobile left TBD.
- Testing stack definition: cargo test + rstest + mockall (backend unit), axum-test +
  testcontainers (backend integration), elm-test (frontend unit), Playwright (E2E).
- `.gitignore` ignoring the reference system `colab-life-test/` and Rust/editor artifacts.
- This `CHANGELOG.md`.
- Development environment scaffold (structure and manifests only, no feature code):
  - Backend Cargo workspace (`backend/`) with crates `api`, `entity`, `migration`,
    `service`; pinned deps centralized in `[workspace.dependencies]`; committed
    `Cargo.lock`. Migrations run via `cargo run -p migration` (no global sea-orm-cli).
  - Frontend Elm app (`frontend/`) with `elm.json`, `elm-test`, and `elm-format` pinned.
  - E2E suite (`e2e/`) with Playwright and a multi-browser `playwright.config.ts`.
  - `rust-toolchain.toml` (1.95.0), `docker-compose.yml` (PostgreSQL 16.14), and a
    `justfile` (`setup`/`test`/`run`/`migrate`/`fmt`/`db-up`/`db-down`).
  - `CONTRIBUTING.md` with the pinned-version table and reproducible setup steps.
  - `.gitignore` extended for Node, Elm, Playwright, and build artifacts.
- Backend walking skeleton: `api` crate now exposes `build_router` and two probes —
  `GET /health` (liveness, no database) and `GET /health/ready` (readiness, pings
  PostgreSQL via SeaORM, `200`/`503`). Covered by a unit test (mock connection, no Docker)
  and an integration test (throwaway PostgreSQL via testcontainers). The `api` crate now
  depends on `sea-orm`; `main` wires tracing, the database connection, and the HTTP server.
- Migration infrastructure split into `PublicMigrator` (cross-tenant `public` schema, also
  driven by the `cargo run -p migration` CLI) and `TenantMigrator` (per-tenant schema, run
  by the future provisioning flow).
- First public-schema migration and entity: `organizations` (tenant root — id, unique name
  doubling as schema slug, plan, due date, employee limit, active flag, timestamps). Covered
  by integration tests (persist + default columns, unique-name constraint). Enabled SeaORM
  `with-uuid`/`with-chrono` and pinned `uuid` (v4) in the workspace.
- `users` table and entity in the public schema: login identity (id, name, globally unique
  email, password_hash, is_admin, soft-delete) belonging to an organization via a
  `organization_id` foreign key. Covered by integration tests (defaults + ownership, unique
  email, foreign-key enforcement).
- Password hashing in the `service` crate: `hash_password`/`verify_password` over Argon2
  (PHC strings, per-password random salt). Unit-tested (round-trip, wrong-password rejection,
  salt randomness, malformed-hash handling); no Docker required. Pinned `argon2` in the
  workspace.
- Tenant provisioning in the `service` crate (`provision_organization`): validates the
  organization name as a safe schema slug, creates and migrates the tenant's dedicated
  PostgreSQL schema (`TenantMigrator` over a `search_path` connection), and persists the
  organization plus its Argon2-hashed admin in one public-schema transaction. Cross-step
  atomicity is best-effort (see ADR 0007). Integration-tested end to end (schema created and
  migrated, admin loggable, duplicate and unsafe names rejected).
- `POST /organizations` endpoint exposing tenant provisioning over HTTP: `201` with the
  created organization and admin, `400` for an invalid name, `409` for a duplicate, `500`
  otherwise. `AppState` now carries the `database_url` alongside the connection (so handlers
  can open search-path connections); `build_router` takes `(db, database_url)`.
  Integration-tested with `axum-test`.
- Native authentication in the `service` crate (`auth`): `authenticate` verifies an
  email/password against the `public` schema (unknown email, soft-deleted user and wrong
  password all collapse to one `InvalidCredentials` outcome to avoid user enumeration; an
  inactive organization yields `OrganizationInactive`), and `encode_token` / `decode_token`
  issue and validate stateless **JWT** session tokens (HS256, `Claims { sub, org, schema,
  is_admin, exp }`, 24h expiry; see ADR 0008). Pinned `jsonwebtoken` (pure-Rust
  `rust_crypto` backend) in the workspace. Unit-tested (token round-trip, wrong secret,
  expiry) and integration-tested (`authenticate` against a real database).
- `POST /auth/login` endpoint exchanging credentials for a session JWT: `200` with
  `{ token, token_type }`, `401` for invalid credentials, `403` for an inactive organization.
  `AppState` now also carries the `jwt_secret`; `build_router` takes
  `(db, database_url, jwt_secret)` and the API reads `JWT_SECRET` from the environment
  (dev-only default in the `justfile`). Integration-tested with `axum-test`.
- Per-request tenant schema resolution in the `service` crate (`tenant::TenantRegistry`):
  caches one `DatabaseConnection` per tenant schema (each pinned via
  `set_schema_search_path`), resolving them by name on demand (see ADR 0009). The schema-name
  validator is now shared between the registry and the provisioner (`is_valid_schema_name`).
  Integration-tested (connection pinned to the requested schema, cache reuse, invalid name
  rejected).
- Authentication extractor `TenantContext` (`api::extract`, `FromRequestParts`): turns an
  `Authorization: Bearer <jwt>` header into verified `Claims` plus a connection to the caller's
  tenant schema (resolved via the `TenantRegistry`). Missing/malformed/invalid/expired token →
  `401`; an unreachable tenant → `500`. Handlers opt into auth by taking it as an argument;
  routes that omit it stay public. `AppState`/`build_router` now build and carry the
  `TenantRegistry` (see ADR 0009). Unit-tested (bearer parsing/validation, no Docker).
- `GET /auth/me`: first protected route, returns the caller's identity
  (`{ user_id, organization_id, schema, is_admin }`) from the verified token. Integration-tested
  with `axum-test` (valid token → identity, missing token → `401`, wrong-secret token → `401`).
- RBAC schema in each tenant: `TenantMigrator` now creates the six `permission_*` tables
  (`resources`, `tasks`, `task_resources`, `profiles`, `profile_tasks`, `profile_users`),
  modeling user → profile → task → resource (see ADR 0010). Resources use `domain.action`
  identifiers; `permission_profile_users.user_id` references `public.users` by value (no
  cross-schema FK). SeaORM entities added under `entity::permission`. Integration-tested
  (the tables land in the migrated tenant schema).
- RBAC seeding on provisioning (`service::permission`): a `Resource` enum defines the minimal
  catalog (`user.*`, `profile.*`) and `seed_tenant_rbac` plants it plus an "administrator"
  profile granting every resource, linking it to the tenant's admin. Provisioning now drops
  the tenant schema on any failure after its creation (not just migration). Unit-tested
  (catalog identifiers/uniqueness) and integration-tested (catalog seeded, admin holds the
  profile).
