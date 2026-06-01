# 0010. Granular RBAC (profile → task → resource)

- **Status:** Accepted
- **Date:** 2026-06-01

## Context

The foundation needs authorization: deciding whether an authenticated caller
([ADR 0008](0008-stateless-jwt-sessions.md), [ADR 0009](0009-per-request-tenant-resolution.md))
may perform a given action. The legacy system models this per tenant with a
three-level chain — a user holds **profiles**, a profile groups **tasks**, a task
groups **resources** (the protected actions) — plus an `isAdmin` shortcut that
grants everything. Resources are named with verbose URIs
(`res://br.com.dgsys.<domain>.<action>`), and the user↔profile join stores the
user id without a cross-schema foreign key.

We redesign freely but keep the granular shape, and must decide: where the tables
live, how resources are identified, how much to seed up front (no domain modules
exist yet), and how a route declares the permission it needs.

## Decision

**Three-level model in the tenant schema.** Six tables, created by
`TenantMigrator` inside each tenant's schema: `permission_resources`,
`permission_tasks`, `permission_task_resources`, `permission_profiles`,
`permission_profile_tasks`, `permission_profile_users`. A user's permissions are
the resources reachable via `profile_users → profiles → profile_tasks → tasks →
task_resources → resources`.

- **Resource identifiers are `domain.action`** (e.g. `user.read`), simplifying the
  legacy `res://…` URIs. They are modeled as a Rust `Resource` enum (type-safe at
  the guard) persisted as the `name` string.
- **`permission_profile_users.user_id` has no cross-schema foreign key** — it
  references `public.users` by value, keeping tenant schemas decoupled from
  `public` (the legacy system does the same).
- **Minimal seed, grown per domain.** Provisioning seeds only the resources that
  exist today (user and RBAC management) and an "Administrator" profile linked to
  the admin user. Each future domain adds its own resources in its own phase.
- **Admin bypass via the token.** A caller whose JWT claim `is_admin` is true is
  authorized without a database lookup; everyone else is checked against the
  chain.
- **Enforcement is explicit in the handler**: `TenantContext::require(resource)`
  returns `403` when the permission is absent. (Implemented in a later step.)

## Consequences

- Fine-grained, composable permissions from the foundation; new domains plug in by
  adding resources and wiring them to tasks/profiles.
- The check runs against the tenant connection the extractor already resolved; the
  admin shortcut keeps the common case query-free.
- No cross-schema FK means orphan `user_id`s are possible (e.g. a deleted user);
  integrity is enforced in application logic, not the database.
- The three-level indirection (task between profile and resource) is more complex
  than a direct profile→resource grant, but matches the legacy model and allows
  reusable task bundles.
- A growing resource catalog must be seeded into every existing tenant as it
  expands; back-filling new resources into provisioned tenants is a concern to
  handle when the catalog grows (deferred).
