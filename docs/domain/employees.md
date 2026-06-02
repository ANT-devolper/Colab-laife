# Employees

> **Status:** ✅ core implemented — `sector`, `role` and `collaborator` exist with RBAC-guarded
> CRUD. The org-hierarchy/"accessible collaborators" service is 🚧 deferred (only `manager_id`
> exists for now).

## Purpose

Maintain the directory of people in an organization and their lifecycle — the core entity
most other modules attach to (notes, feedback, DISC, tasks).

## Key concepts / entities

- **Sector** — an organizational unit (department) inside a tenant. ✅ Implemented as the
  `sector` table in the tenant schema (`id`, `name`, `active`, timestamps), with RBAC-guarded
  CRUD at `/sectors` (`sector.{read,create,update,delete}`); removal is a soft delete. The Elm
  SPA offers full write for sectors (create / inline rename / deactivate) via `Page/Sectors.elm`.
- **Role** (`role`) — a job title with the legacy description fields. ✅ Implemented as the
  `role` table in the tenant schema (`name` plus the optional `profile_suggestion`,
  `objective`, the `requirement_*` breakdown and `observation`, `active`, timestamps), with
  RBAC-guarded CRUD at `/roles` (`role.{read,create,update,delete}`); removal is a soft delete.
- **Collaborator** — a person managed inside a tenant. ✅ Implemented as the `collaborator`
  table in the tenant schema. Distinct from **user** (a login identity in `public.users`); the
  collaborator↔user link is by value (`user_id`, no cross-schema FK), and a collaborator
  references its sector, role and manager (self-FK). Carries `whatsapp`, `email`, `is_manager`,
  `date_of_hire` and `active`. RBAC-guarded CRUD at `/collaborators`
  (`collaborator.{read,create,update,delete}`); create/update reject dangling references with
  `422`; removal is a soft delete.

## Main flows

- Create / edit an employee.
- List and search the directory.
- Deactivate / offboard an employee (lifecycle).

## Permissions / roles

- Managed by organization admins/HR; scoping defined at design time.
- The organization's `employee_limit` (on `public.organizations`) caps headcount.

## Status

Core implemented. The module's tables live in the tenant schema (`TenantMigrator`): `sector`,
`role` and `collaborator` exist with their RBAC-guarded CRUD. Still deferred: the
org-hierarchy/"accessible collaborators" service (only the `manager_id` column exists for now),
write CRUD from the Elm UI, and multi-company (`company_info`/`company_id`).

## Reference

See the corresponding feature in `colab-life-test/` (read-only, git-ignored) for the
legacy behavior.
