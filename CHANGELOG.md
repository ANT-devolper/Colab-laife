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
- RBAC enforcement: `service::permission::has_permission` walks
  `profile_users → profile_tasks → task_resources → resources` in the tenant schema, and the
  `TenantContext::require(resource)` guard authorizes a route (admins bypass via the token;
  others are checked, `403` when not granted). `GET /users` is the first guarded route —
  requires `user.read` and lists the caller organization's users. Integration-tested
  (`has_permission` for a linked admin vs an unlinked user; `GET /users` `200` for the admin,
  `403` for a permissionless member). Completes the Phase 1 foundation (see ADR 0010).
- Sectors, the first tenant-domain resource (Phase 2): `TenantMigrator` now creates the
  `sector` table (id, name, `active` soft-delete flag, timestamps) and `entity::sector` maps
  it. The `Resource` catalog gains `sector.{read,create,update,delete}` (auto-granted to the
  seeded administrator profile), and `api::sectors` exposes RBAC-guarded CRUD over the
  caller's tenant schema (`ctx.tenant_db`): `GET`/`POST /sectors`, `PATCH`/`DELETE
  /sectors/{id}`, with removal as a soft delete (`active = false`) and lists filtered to
  active rows. This is the first domain route querying the tenant schema instead of `public`.
  Integration-tested (admin create → list → update → soft delete; permissionless member →
  `403`) plus a migration test (the `sector` table lands in the migrated tenant schema).
- Roles, the second tenant-domain resource (Phase 2): `TenantMigrator` now creates the
  `role` table with the full legacy description set (`name` plus optional `profile_suggestion`,
  `objective`, the `requirement_*` breakdown — education, experience, attention, knowledge,
  skill, attitude, delivery — and `observation`), an `active` soft-delete flag and timestamps;
  `entity::role` maps it. The `Resource` catalog gains `role.{read,create,update,delete}`
  (auto-granted to the seeded administrator profile), and `api::roles` exposes RBAC-guarded CRUD
  over the caller's tenant schema: `GET`/`POST /roles`, `PATCH`/`DELETE /roles/{id}`. `PATCH`
  writes only the fields present in the body (an omitted field is left untouched); removal is a
  soft delete and lists are filtered to active rows. Integration-tested (admin create with
  description fields → list → partial update preserving untouched fields → soft delete;
  permissionless member → `403`) plus a migration test (the `role` table lands in the migrated
  tenant schema).
- Collaborators, the third tenant-domain resource (Phase 2): `TenantMigrator` now creates the
  `collaborator` table — the corporate record of a person inside a tenant (`name`, optional
  `sector_id`/`role_id` FKs, an optional `manager_id` self-FK for the hierarchy, `whatsapp`,
  `email`, `is_manager`, `date_of_hire`, an `active` soft-delete flag and timestamps). The
  optional `user_id` links a collaborator to its `public.users` login **by value** (no
  cross-schema FK, like `permission_profile_users`). `entity::collaborator` maps it with
  relations to sector, role and the self-referencing manager. The `Resource` catalog gains
  `collaborator.{read,create,update,delete}` (auto-granted to the seeded administrator profile),
  and `api::collaborators` exposes RBAC-guarded CRUD over the caller's tenant schema:
  `GET`/`POST /collaborators`, `PATCH`/`DELETE /collaborators/{id}`. `create`/`update` validate
  that any referenced sector/role/manager points at an existing active row (a dangling reference
  → `422`); `PATCH` writes only the fields present in the body, and removal is a soft delete. The
  org-hierarchy/"accessible collaborators" service is deferred — only the `manager_id` column
  lands now. Integration-tested (admin create manager → report referencing it → list → partial
  update preserving untouched FKs → soft delete; unknown sector and unknown manager → `422`;
  permissionless member → `403`) plus a migration test (the `collaborator` table lands in the
  migrated tenant schema).
- Elm SPA foundation (Phase 2): the frontend (`frontend/`) gains a sign-in page built on The
  Elm Architecture — `Api.elm` (the HTTP boundary: login request encoder, `{ token, token_type }`
  response decoder, `Authorization: Bearer` header helper, `POST /auth/login` command, all
  root-relative since the SPA is same-origin), `Page/Login.elm` (the credentials form, reporting
  the obtained token through an `OutMsg`) and `Main.elm` (the `Login` / `Authenticated` shell).
  Added `elm/http` and `elm/json` to `elm.json`. Unit-tested with the first `elm-test`
  (`tests/ApiTest.elm`: the login encoder and the response decoder, including a missing-field
  failure).
- Single-origin SPA delivery (Phase 2, [ADR 0011](docs/adr/0011-serve-spa-from-axum.md)): the
  `api` binary now serves the built Elm SPA itself via `api::with_static_spa` (a `tower-http`
  `ServeDir` fallback with `index.html` for unknown client-side routes), so the browser talks to
  one origin with no CORS. API routes keep precedence over static files. `main` enables it only
  when `FRONTEND_DIST` is set (API-only otherwise), and the `justfile` gains `frontend-build`
  (compile `src/Main.elm` into `frontend/dist` plus the HTML shell) with `run` pointing
  `FRONTEND_DIST` at it. New workspace dependency `tower-http` (feature `fs`). Integration-tested
  with a `MockDatabase` (no Docker): index at the root, real assets, unknown-path fallback to
  `index.html`, and API precedence.
- Read-only directory in the Elm SPA (Phase 2): after login, `Page/Directory.elm` fetches
  `/collaborators`, `/sectors` and `/roles` with the Bearer token and renders a table per list,
  each with its own loading / empty / error state. `Api.elm` gains the `Sector`/`Role`/
  `Collaborator` types, their decoders and the authenticated `GET` helpers; `Main.elm` now
  transitions from login to the directory. Unit-tested with `elm-test` (the new decoders,
  including a null `email` and an empty list).
- First Playwright end-to-end test (Phase 2): `e2e/tests/login.spec.ts` drives the real stack —
  Playwright's `webServer` (`e2e/scripts/e2e-stack.mjs`) brings up a dedicated PostgreSQL
  (via `docker run`, reset each run), applies the public migrations, builds the SPA and runs the
  API serving it on a single origin. The test provisions a tenant through the API, signs in
  through the SPA and asserts the empty directory. The chromium project is the gate
  (`npm test` / `just e2e`); Firefox/WebKit are available via `npm run test:all` once their
  system libraries are installed. The `justfile` gains an `e2e` recipe.
- Feedback, the first people-domain resource (Phase 3A): `TenantMigrator` now creates the
  `feedback` table — a structured feedback event about a collaborator (`collaborator_id` FK,
  `feedback_date`, optional `next_feedback_date`, the expectation-contract observations
  `expectation_contract_observation`/`…_private`, `status`, an `active` soft-delete flag and
  timestamps). Redesigned from the legacy model: manager/sector are **not** stored (they are
  derived from the collaborator at read time) and AI/transcription is out of scope.
  `entity::feedback` maps it with a relation to collaborator. The `Resource` catalog gains
  `feedback.{read,create,update,delete}` (auto-granted to the seeded administrator profile), and
  `api::feedback` exposes RBAC-guarded CRUD over the caller's tenant schema: `GET`/`POST
  /feedbacks`, `PATCH`/`DELETE /feedbacks/{id}`. `GET` lists active feedback newest-first and
  accepts an optional `?collaborator_id=` filter; `create` rejects an unknown/inactive
  collaborator with `422`; `PATCH` writes only the fields present in the body; removal is a soft
  delete. Integration-tested (admin create → list filtered by collaborator → partial update →
  soft delete; unknown collaborator → `422`; permissionless member → `403`) plus a migration
  test (the `feedback` table lands in the migrated tenant schema).
- Expectation-contract items (Phase 3A): `TenantMigrator` now creates the
  `expectation_contract_item` table — a checklist entry of a feedback's expectation contract
  (`feedback_id` FK, a `kind` discriminator `goal`/`behavior`, `description`, `done`, an `active`
  soft-delete flag and timestamps). Redesigned from the legacy model, which split this into two
  identical tables (`expectation_contract_goals` and `expectation_contract_behavior`); we unify
  them with `kind`. `entity::expectation_contract_item` maps it with a relation to feedback. The
  `Resource` catalog gains `expectation.{read,create,update,delete}` (auto-granted to the seeded
  administrator profile), and `api::expectation_items` exposes RBAC-guarded CRUD over the tenant
  schema: `GET`/`POST /expectation-items`, `PATCH`/`DELETE /expectation-items/{id}`. `GET`
  accepts optional `?feedback_id=` and `?kind=` filters; `create` rejects an invalid `kind` or an
  unknown feedback with `422`; `PATCH` writes only the fields present in the body; removal is a
  soft delete. Integration-tested (admin create goal + behavior → list filtered by feedback and
  by kind → mark done → soft delete; invalid kind → `422`; unknown feedback → `422`;
  permissionless member → `403`) plus a migration test.
- Feedback behaviors (Phase 3A): `TenantMigrator` now creates the `feedback_behavior` table — a
  scored DISC-values line of a feedback (`feedback_id` FK, required `value_description` and
  `behavior_description`, optional `behavior_obs`/`value_instruction`, an integer `score`, an
  `active` soft-delete flag and timestamps); `entity::feedback_behavior` maps it with a relation
  to feedback. The `Resource` catalog gains `feedback_behavior.{read,create,update,delete}`
  (auto-granted to the seeded administrator profile), and `api::feedback_behaviors` exposes
  RBAC-guarded CRUD over the tenant schema: `GET`/`POST /feedback-behaviors`, `PATCH`/`DELETE
  /feedback-behaviors/{id}`. `GET` accepts an optional `?feedback_id=` filter; `create` rejects
  an unknown feedback with `422`; `PATCH` writes only the fields present in the body; removal is a
  soft delete. Integration-tested (admin create → list filtered by feedback → partial update →
  soft delete; unknown feedback → `422`; permissionless member → `403`) plus a migration test.
- Annotations (Phase 3A): `TenantMigrator` now creates the `annotation` table — a quick scored
  note about a collaborator (`collaborator_id` FK, `note_date`, a primary score
  `score1_number`/`score1_type` plus optional description, an optional second score, an
  `ask_amount_days`/`amount_days` pair, a free `main_note`, `period_start_date`, `observation`,
  `recorded_on_mobile`, an `active` soft-delete flag and timestamps). Redesigned from the legacy
  model: manager is derived from the collaborator at read time, and the deferred concerns are
  dropped — `company_id` (multi-company), attachments (S3) and feedback messaging
  (notifications). `entity::annotation` maps it with a relation to collaborator. The `Resource`
  catalog gains `annotation.{read,create,update,delete}` (auto-granted to the seeded
  administrator profile), and `api::annotations` exposes RBAC-guarded CRUD over the tenant
  schema: `GET`/`POST /annotations`, `PATCH`/`DELETE /annotations/{id}`. `GET` lists newest-first
  with an optional `?collaborator_id=` filter; `create` rejects an unknown collaborator with
  `422`; `PATCH` writes only the fields present in the body; removal is a soft delete.
  Integration-tested (admin create → list filtered by collaborator → partial update → soft
  delete; unknown collaborator → `422`; permissionless member → `403`) plus a migration test.
  This completes the Phase 3A backend (feedback + expectation contract + behaviors + annotations).
- Sector write UI in the Elm SPA: the authenticated shell now manages sectors end to end —
  `Page/Sectors.elm` lists them and offers a create form, inline rename and deactivate,
  re-fetching the list after each successful mutation. `Api.elm` gains `createSector`/
  `updateSector`/`deleteSector` (over a shared authenticated-request helper) and the pure
  `encodeSectorForm`; `Main.elm` composes `Page.Sectors` with the still read-only
  `Page.Directory` (collaborators and roles). Unit-tested with `elm-test` (the sector encoder)
  and end-to-end with Playwright (`e2e/tests/sectors-write.spec.ts`: sign in → create → inline
  rename → deactivate). The backend is unchanged (the `/sectors` write routes already existed).
- Role write UI in the Elm SPA: `Page/Roles.elm` lists roles and offers a single form that
  carries the full legacy field set (name plus the optional `profile_suggestion`, `objective`
  and `requirement_*`/`observation` text areas), serving both create and edit (pre-filled from
  the row), plus deactivate; the list is re-fetched after each successful mutation. `Api.elm`
  gains `createRole`/`updateRole`/`deleteRole`, the pure `encodeRoleForm` (omits blank optional
  fields) and `roleFormFromRole`, and its `roleDecoder`/`Role` now carry every description
  field so an edit can pre-fill. `Main.elm` composes `Page.Roles`; `Page.Directory` is now
  collaborators-only. Unit-tested with `elm-test` (the role encoder and the extended decoder)
  and end-to-end with Playwright (`e2e/tests/roles-write.spec.ts`: sign in → create → edit →
  deactivate). The backend is unchanged (the `/roles` write routes already existed).
- Collaborator write UI in the Elm SPA, closing the cadastro write CRUD: `Page/Collaborators.elm`
  fetches collaborators, sectors and roles, and offers a single form (create + edit + deactivate)
  whose sector/role/manager fields are dropdowns populated from the active lists, plus
  WhatsApp/email and an "is manager" checkbox; a failed save (e.g. the backend's `422` for a
  dangling reference) surfaces as a form error. `Api.elm` gains
  `createCollaborator`/`updateCollaborator`/`deleteCollaborator`, the pure
  `encodeCollaboratorForm` (always sends name + `is_manager`, omits unset references/contacts)
  and `collaboratorFormFromCollaborator`, and its `Collaborator`/`collaboratorDecoder` now carry
  the references and WhatsApp so an edit can pre-fill. `Page.Directory` is removed — its
  collaborators list is superseded by the write page — and `Main.elm` composes the three cadastro
  pages (`Sectors`/`Roles`/`Collaborators`). Unit-tested with `elm-test` (the collaborator encoder
  and the extended decoder) and end-to-end with Playwright (`e2e/tests/collaborators-write.spec.ts`:
  seed a sector + role via the API, sign in → create picking them → edit → deactivate). The backend
  is unchanged (the `/collaborators` write routes already existed).
- Feedback UI in the Elm SPA (Phase 3A, feedback parent): the authenticated shell now has tabs
  ("Cadastro" / "Feedback"). `Page/Feedback.elm` picks a collaborator from a dropdown, lists that
  collaborator's feedbacks and manages them with a single create/edit form (date, next date,
  status, public/private observations) plus deactivate, re-fetching after each mutation. `Api.elm`
  gains `Feedback`/`feedbackDecoder`, `getFeedbacks` (with the `?collaborator_id=` filter),
  `createFeedback`/`updateFeedback`/`deleteFeedback` and the pure `encodeFeedbackForm` (converts
  the `YYYY-MM-DD` date inputs to RFC3339). Unit-tested with `elm-test` (the feedback decoder and
  encoder) and end-to-end with Playwright (`e2e/tests/feedback-write.spec.ts`: seed a collaborator
  via the API, sign in → Feedback tab → pick collaborator → create → edit status → deactivate). The
  backend is unchanged (the `/feedbacks` routes already existed).
- Expectation-contract UI nested in an open feedback (Phase 3A): the Feedback page gains an "Open"
  action that expands a feedback into two checklists (goals, behaviors); items can be added,
  toggled (`done`) and removed. `Api.elm` gains `ExpectationItem`/`expectationItemDecoder`,
  `getExpectationItems` (the `?feedback_id=` filter), `createExpectationItem`/
  `updateExpectationItem`/`deleteExpectationItem` and the pure `encodeExpectationItemForm`.
  Unit-tested with `elm-test` (decoder + encoder) and exercised end-to-end in
  `e2e/tests/feedback-write.spec.ts` (open feedback → add a goal → mark it done → remove it). The
  backend is unchanged (the `/expectation-items` routes already existed).
- Scored-behaviors UI nested in an open feedback (Phase 3A): the contract section gains a
  "Scored behaviors" table with a create/edit form (value/behavior descriptions, optional
  observation/instruction, integer score) and remove. `Api.elm` gains
  `FeedbackBehavior`/`feedbackBehaviorDecoder`, `getFeedbackBehaviors` (the `?feedback_id=`
  filter), `createFeedbackBehavior`/`updateFeedbackBehavior`/`deleteFeedbackBehavior` and the
  pure `encodeFeedbackBehaviorForm` (always sends the required fields and the integer score).
  Unit-tested with `elm-test` (decoder + encoder) and exercised end-to-end in
  `e2e/tests/feedback-write.spec.ts` (add a scored behavior → edit its score → remove it). The
  backend is unchanged (the `/feedback-behaviors` routes already existed). The annotations UI is
  next.
