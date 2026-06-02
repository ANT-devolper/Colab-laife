# Notes & feedback

> **Status:** ✅ backend implemented — the `feedback` event, its expectation-contract items, its
> scored behaviors and standalone `annotation` notes all have RBAC-guarded CRUD. The Elm UI is
> 🚧 in progress: the **feedback parent** (list/create/edit/deactivate per collaborator) is done;
> the **expectation-contract items** (goals/behaviors checklist) and the **scored behaviors** of
> an open feedback are done too; the annotations UI is next. The deferred concerns below
> (AI/transcription, attachments, messaging) remain planned.

## Purpose

Let people record continuous notes and exchange structured feedback (manager-to-report) about
collaborators, instead of relying only on annual reviews.

## Key concepts / entities

- **Feedback** — ✅ a structured feedback event about a **collaborator**, implemented as the
  `feedback` table in the tenant schema (`collaborator_id` FK, `feedback_date`, optional
  `next_feedback_date`, the expectation-contract observations — public and private —, a free
  `status`, `active`, timestamps). Manager and sector are **not** stored; they are derived from
  the collaborator at read time. RBAC-guarded CRUD at `/feedbacks`
  (`feedback.{read,create,update,delete}`); the list accepts an optional `?collaborator_id=`
  filter and `create` rejects an unknown collaborator with `422`; removal is a soft delete.
- **Expectation contract** — ✅ per-feedback checklist of goals and behaviours, implemented as
  the `expectation_contract_item` table (one row per item, a `kind` discriminator `goal`/
  `behavior`, `description`, `done`, `active`). The legacy model used two identical tables; we
  unify them with `kind`. RBAC-guarded CRUD at `/expectation-items`
  (`expectation.{read,create,update,delete}`), with optional `?feedback_id=`/`?kind=` filters;
  removal is a soft delete.
- **Feedback behaviour** — ✅ the DISC-values scoring lines of a feedback, implemented as the
  `feedback_behavior` table (`value_description`, `behavior_description`, optional `behavior_obs`/
  `value_instruction`, integer `score`, `active`). RBAC-guarded CRUD at `/feedback-behaviors`
  (`feedback_behavior.{read,create,update,delete}`), with an optional `?feedback_id=` filter;
  removal is a soft delete.
- **Annotation (note)** — ✅ a quick scored note about a collaborator, implemented as the
  `annotation` table (`note_date`, a primary score `score1_number`/`score1_type` + optional
  second score, an `ask_amount_days`/`amount_days` pair, `main_note`, `period_start_date`,
  `observation`, `recorded_on_mobile`, `active`). Manager is derived from the collaborator at
  read time. RBAC-guarded CRUD at `/annotations` (`annotation.{read,create,update,delete}`), with
  an optional `?collaborator_id=` filter; removal is a soft delete. Attachments (S3) and feedback
  messaging (notifications) stay deferred.

## Main flows

- Create / edit / remove a feedback for a collaborator (subject to permissions). ✅ (backend +
  Elm UI: the Feedback tab picks a collaborator and manages their feedbacks.)
- List the feedback for a collaborator over time (newest first). ✅
- Author quick notes about a collaborator (and list/edit/remove them). ✅ (backend; Elm UI 🚧.)

## Permissions / roles

- Guarded by the `feedback.*` resources (RBAC); the seeded administrator profile holds them.
- Finer visibility rules (e.g. the private observation visible only to managers) are deferred.

## Deferred (out of scope for now)

- AI/transcription of feedback recordings (`feedback_record`, OpenAI scripts).
- Attachments on notes (needs the `Storage` trait, Phase 7).
- Feedback messaging/notifications (needs the `Mailer`/notifications crosscut, Phase 7).

## Reference

See the corresponding feature in `colab-life-test/` (read-only, git-ignored) for the
legacy behavior.
