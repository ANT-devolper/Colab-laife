# 0007. Tenant provisioning and schema resolution

- **Status:** Accepted
- **Date:** 2026-06-01

## Context

With schema-per-tenant ([ADR 0004](0004-multi-tenant-schema-per-tenant.md)), creating a
tenant means more than inserting a row: a dedicated PostgreSQL schema must exist and be
migrated. We need a concrete, testable mechanism for that, and a safe way to point a
connection at a specific tenant schema (the same mechanism the future per-request resolver
will use). The organization `name` doubles as the schema slug, so it reaches DDL where
identifiers cannot be parameterized.

## Decision

Provisioning lives in `service::provisioning::provision_organization(db, database_url, input)`:

1. **Validate the name** as a safe SQL identifier (`^[a-z][a-z0-9_]{0,62}$`) and reject
   anything else, since it is interpolated into `CREATE SCHEMA`.
2. **Reject duplicates** by pre-checking `organizations.name` (the unique constraint is the
   backstop).
3. **Create and migrate the schema:** `CREATE SCHEMA`, then open a dedicated connection with
   SeaORM `ConnectOptions::set_schema_search_path(name)` and run `TenantMigrator` against it.
   Setting `search_path` on the connection is how we target a tenant schema; the per-request
   resolver (planned) will use the same option.
4. **Persist identity:** write the organization and its admin user (Argon2 hash) in a single
   `public`-schema transaction.

## Consequences

- A clear, integration-tested path (testcontainers) from "no tenant" to "schema migrated +
  admin able to log in".
- `database_url` must be available to the provisioning caller (to open the search-path
  connection); the HTTP layer will carry it in shared state.
- Cross-step atomicity is **best-effort**, not a single transaction: schema creation/migration
  is non-transactional DDL on a separate connection. A failed migration drops the new schema;
  a later failure can still leave an orphan schema. Hardening (full cleanup/idempotent retry)
  is deferred.
- Name validation constrains tenant slugs to lowercase identifiers — acceptable given the
  name is also a schema name.
