# DISC assessment

> **Status:** 🚧 in progress — the collaborator result backend is ✅ implemented (table + entity +
> RBAC-guarded CRUD + profile derivation). The public questionnaire submission, the Elm
> questionnaire/results UI and the recruitment-candidate variant are next.

## Purpose

Administer the DISC behavioral questionnaire to employees and produce a profile across the
four DISC dimensions: **D**ominance, **I**nfluence, **S**teadiness, **C**onscientiousness.

## Key concepts / entities

- **Questionnaire** — the set of DISC questions/items presented to a respondent. In our redesign
  the 40 items are not persisted; they live in the Elm questionnaire (as in the legacy), and only
  the scores are stored. 🚧 planned.
- **Response** — an employee's answers to a questionnaire instance (client-side only; not stored).
- **Profile / result** — ✅ implemented as the `collaborator_disc_result` table in the tenant
  schema (`collaborator_id` FK + the four dimension scores `executor`/`communicator`/`planner`/
  `analyst`, timestamps). Results are kept as history (no soft-delete); reads return newest-first.
  The **primary/secondary profile is derived at read time** by `service::disc::profile` (highest
  and second-highest dimension; ties break by a fixed order), not stored. RBAC-guarded CRUD at
  `/disc-results` (`disc.{read,create,delete}`); `create` rejects an unknown collaborator with
  `422`; results are immutable (no update) and `delete` is a hard delete.

## Main flows

- Invite/assign an employee to take the assessment.
- Respondent completes the questionnaire.
- System scores the response and stores the resulting profile.
- View an employee's DISC profile.

## Permissions / roles

- Respondents complete their own assessment.
- Who may view results (the employee, their manager, admins) is defined at design time.

## Status

Backend for collaborator results implemented (`collaborator_disc_result` in `TenantMigrator`,
`entity::collaborator_disc_result`, `service::disc`, `api::disc_results`). The scoring (ranking
1–4 per row → points `4 - position` summed per dimension) lives in the Elm questionnaire and is
🚧 next, along with the public (no-auth, UUID-in-URL) submission endpoint and the
recruitment-candidate result variant.

## Reference

See the corresponding feature in `colab-life-test/` (read-only, git-ignored) for the
legacy behavior and scoring.
