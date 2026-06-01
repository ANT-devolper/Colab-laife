# 0003. Persistence with SeaORM over PostgreSQL

- **Status:** Accepted
- **Date:** 2026-06-01

## Context

The backend ([0002](0002-backend-rust-axum.md)) needs persistence for a relational,
multi-tenant data model ([0004](0004-multi-tenant-schema-per-tenant.md)) with versioned
schema changes.

## Decision

Use **PostgreSQL** as the database and **SeaORM** (async ORM) as the data-access layer,
with schema changes managed by **`sea-orm-migration`**. Migrations run via the `migration`
crate's binary (`cargo run -p migration` / `just migrate`) — no global `sea-orm-cli`
required for day-to-day work.

## Consequences

- Async, type-safe queries that fit the Tokio/Axum stack.
- PostgreSQL schemas underpin the per-tenant isolation model.
- Migrations are code, versioned and reproducible; integration tests pin the same Postgres
  version (testcontainers).
- Services are kept independent of the ORM where possible (mockable) so business logic is
  unit-testable without a database.
