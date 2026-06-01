# Employees

> **Status:** 🚧 partially implemented — `sector` and `role` exist; `collaborator` is planned.

## Purpose

Maintain the directory of people in an organization and their lifecycle — the core entity
most other modules attach to (notes, feedback, DISC, tasks).

## Key concepts / entities

- **Sector** — an organizational unit (department) inside a tenant. ✅ Implemented as the
  `sector` table in the tenant schema (`id`, `name`, `active`, timestamps), with RBAC-guarded
  CRUD at `/sectors` (`sector.{read,create,update,delete}`); removal is a soft delete.
- **Role** (`role`) — a job title with the legacy description fields. ✅ Implemented as the
  `role` table in the tenant schema (`name` plus the optional `profile_suggestion`,
  `objective`, the `requirement_*` breakdown and `observation`, `active`, timestamps), with
  RBAC-guarded CRUD at `/roles` (`role.{read,create,update,delete}`); removal is a soft delete.
- **Collaborator** — a person managed inside a tenant. Distinct from **user** (a login
  identity in `public.users`); the collaborator↔user link is by value (`user_id`, no
  cross-schema FK), and a collaborator references its sector, role and manager (self-FK).
  🚧 Planned.

## Main flows

- Create / edit an employee.
- List and search the directory.
- Deactivate / offboard an employee (lifecycle).

## Permissions / roles

- Managed by organization admins/HR; scoping defined at design time.
- The organization's `employee_limit` (on `public.organizations`) caps headcount.

## Status

Partially implemented. The module's tables live in the tenant schema (`TenantMigrator`):
`sector` and `role` exist with their RBAC-guarded CRUD; `collaborator` is next.

## Reference

See the corresponding feature in `colab-life-test/` (read-only, git-ignored) for the
legacy behavior.
