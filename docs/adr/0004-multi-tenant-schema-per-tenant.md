# 0004. Multi-tenancy: schema per tenant

- **Status:** Accepted
- **Date:** 2026-06-01

## Context

ColabLife is a multi-tenant SaaS: each customer organization's data must be isolated from
others. Common options are a shared schema with a tenant discriminator column, a schema per
tenant, or a database per tenant.

## Decision

Use **one PostgreSQL schema per tenant**, with a shared **`public`** schema for cross-tenant
identity. The `public` schema holds `organizations` (tenant root, whose `name` is the tenant
schema slug) and `users` (login identities with a globally unique `email` so login resolves
the tenant). Migrations split into `PublicMigrator` (public schema) and `TenantMigrator`
(per-tenant schema, applied by the tenant provisioning flow).

## Consequences

- Strong data isolation at the schema boundary; per-tenant tables never mix.
- Cross-tenant concerns (auth, billing) live in one place (`public`).
- Provisioning a tenant means creating a schema and running `TenantMigrator` against it.
- Schema migrations must fan out across all tenant schemas — operationally heavier than a
  single shared schema. Accepted as the price of isolation.
- Lighter than database-per-tenant while keeping a clear isolation boundary.
