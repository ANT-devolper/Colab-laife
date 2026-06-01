# Employees

> **Status:** 🚧 planned — not implemented yet.

## Purpose

Maintain the directory of people in an organization and their lifecycle — the core entity
most other modules attach to (notes, feedback, DISC, tasks).

## Key concepts / entities

- **Employee** — a person managed inside a tenant. Distinct from **user** (a login
  identity in `public.users`); the employee↔user mapping is part of this module's design.
- Employee attributes (role, department, status, hire date) are defined when designed.

## Main flows

- Create / edit an employee.
- List and search the directory.
- Deactivate / offboard an employee (lifecycle).

## Permissions / roles

- Managed by organization admins/HR; scoping defined at design time.
- The organization's `employee_limit` (on `public.organizations`) caps headcount.

## Status

Planned. Lives in the tenant schema once the `TenantMigrator` grows tables.

## Reference

See the corresponding feature in `colab-life-test/` (read-only, git-ignored) for the
legacy behavior.
