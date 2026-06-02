# Notes & feedback

> **Status:** 🚧 partially implemented — the `feedback` event and its expectation-contract items
> exist with RBAC-guarded CRUD; `feedback_behavior` and annotations (notes) are planned.

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
- **Feedback behaviour** — 🚧 planned: the DISC-values scoring lines of a feedback
  (`value_description`, `behavior_description`, `score`, …).
- **Annotation (note)** — 🚧 planned: quick notes about a collaborator (scores, a main note,
  observation). Attachments (S3) and feedback messaging (notifications) stay deferred.

## Main flows

- Create / edit / remove a feedback for a collaborator (subject to permissions). ✅
- List the feedback for a collaborator over time (newest first). ✅
- Author quick notes about a collaborator. 🚧

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
