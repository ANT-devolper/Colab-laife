# Domain

The business modules of ColabLife. Each page describes a module's intended behavior using
a common template: **Purpose**, **Key concepts / entities**, **Main flows**,
**Permissions / roles**, **Status**, **Reference**.

> **Status:** all modules below are 🚧 **planned**. The only persisted entities today are
> `organization` and `user` in the cross-tenant `public` schema (see
> [`../architecture.md`](../architecture.md)). Pages are filled in as each module is
> implemented — the documentation directive in [`../../CLAUDE.md`](../../CLAUDE.md) keeps
> this in sync.

## Modules

| Module | Page | Purpose (one line) |
|---|---|---|
| Notes & feedback | [`notes-feedback.md`](notes-feedback.md) | Continuous notes and peer/manager feedback. |
| DISC assessment | [`disc-assessment.md`](disc-assessment.md) | DISC behavioral profile questionnaire and results. |
| Dashboard | [`dashboard.md`](dashboard.md) | Aggregated metrics and overview for the organization. |
| Employees | [`employees.md`](employees.md) | Directory and lifecycle of an organization's people. |
| Tasks | [`tasks.md`](tasks.md) | Task assignment and tracking. |
| Recruitment | [`recruitment.md`](recruitment.md) | Job openings and candidate pipeline. |
| Whistleblower channel | [`whistleblower.md`](whistleblower.md) | Confidential reporting channel. |
| Climate survey | [`climate-survey.md`](climate-survey.md) | Organizational climate / engagement surveys. |

## Glossary

- **Tenant** — an isolated customer of the SaaS, backed by its own PostgreSQL schema.
- **Organization** — the tenant root record (`public.organizations`); its `name` is the
  tenant schema slug.
- **User** — a login identity (`public.users`) belonging to one organization; `email` is
  globally unique. `is_admin` marks organization administrators.
- **Employee** — a person managed inside a tenant (the people the modules act on). Distinct
  from *user* (a login); the mapping between them is defined by the Employees module.
- **DISC** — a behavioral assessment model with four dimensions: **D**ominance,
  **I**nfluence, **S**teadiness, **C**onscientiousness.

## Reference

The previous system in `colab-life-test/` is a **functional behavioral reference only** —
read-only and git-ignored. We redesign data model and API contracts freely; the reference
documents *what the product did*, not how the reimplementation must be built.
