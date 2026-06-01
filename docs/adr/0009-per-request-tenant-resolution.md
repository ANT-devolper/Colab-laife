# 0009. Per-request tenant schema resolution and auth extractor

- **Status:** Accepted
- **Date:** 2026-06-01

## Context

With schema-per-tenant ([ADR 0004](0004-multi-tenant-schema-per-tenant.md)) and
stateless JWT sessions ([ADR 0008](0008-stateless-jwt-sessions.md)), every
authenticated request must (a) prove who the caller is and (b) reach that
tenant's PostgreSQL schema. The session token already carries the tenant
`schema` in its claims, so the open questions are *how to route a request to the
right schema* and *how to wire authentication into Axum handlers*.

The legacy system answered both in one `processRequest` step: it read the token
from an `x-api-key` header, kept a process-wide cache of one connection pool per
schema (`Database.sequelize[schema]`), and re-fetched the user from the database
on every request to check `deleted`.

Options considered for schema routing: a **connection cache keyed by schema**
(each connection's `search_path` pinned once) versus a **single pool with
`SET LOCAL search_path` per request transaction**. For wiring: an Axum
**`FromRequestParts` extractor** the handler asks for, versus a **middleware
layer** injecting context via request extensions.

## Decision

**Schema routing — connection cache per schema.** A `TenantRegistry`
(`service::tenant`) holds the `database_url` and a
`RwLock<HashMap<schema, Arc<DatabaseConnection>>>`. `connection(schema)` validates
the schema name (shared `is_valid_schema_name`, also used by the provisioner),
returns the cached connection, or opens a new one with
`ConnectOptions::set_schema_search_path` — the same mechanism provisioning uses.
A double-checked insert guarantees one connection per schema. The registry does
**not** verify the schema exists (a missing schema surfaces as a later query
error).

**Wiring — `FromRequestParts` extractor.** A `TenantContext { claims, tenant_db }`
extractor reads `Authorization: Bearer <jwt>`, validates it with
`service::auth::decode_token`, and resolves the tenant connection from the
registry. Handlers opt into authentication by taking it as an argument.

**Conventions** (diverging from the legacy system, which used `x-api-key`/403 and
a per-request user re-fetch):

- Token in the standard `Authorization: Bearer` header.
- Missing/malformed/invalid/expired token → **401**. `403` is reserved for the
  upcoming RBAC guard (authenticated but lacking a permission).
- **Stateless**: the extractor trusts the verified token; it does not re-read the
  user on each request (consistent with ADR 0008).

## Consequences

- A request reaches its tenant schema with no extra lookup beyond the (cached)
  connection; the schema travels in the verified token.
- One connection pool per active tenant is held for the process lifetime. This
  mirrors the legacy design and is fine for the expected tenant count; if it ever
  grows large, the cache will need eviction (deferred).
- The extractor composes: the RBAC guard (incremento 8) builds on `TenantContext`
  rather than re-parsing the token.
- Because authentication is stateless, a deleted user or deactivated organization
  keeps access until the token expires (24h). Pre-expiry revocation is deferred.
- A token naming a non-existent schema is only rejected when its first query
  fails, not at extraction time.
