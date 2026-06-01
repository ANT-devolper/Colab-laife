# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
