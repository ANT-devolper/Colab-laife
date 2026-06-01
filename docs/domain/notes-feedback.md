# Notes & feedback

> **Status:** 🚧 planned — not implemented yet.

## Purpose

Let people record continuous notes and exchange structured feedback (peer-to-peer and
manager-to-report) about employees, instead of relying only on annual reviews.

## Key concepts / entities

- **Note** — a free-form entry authored by a user about an employee, with a timestamp.
- **Feedback** — a structured message (e.g. positive / constructive) directed at an
  employee, attributed to its author.

## Main flows

- Author a note or feedback about an employee.
- List the notes/feedback for an employee over time.
- Edit or remove one's own entries (subject to permissions).

## Permissions / roles

- Authors manage their own entries.
- Visibility rules (private to author, visible to managers, visible to the subject) are
  defined when the module is designed.

## Status

Planned. No entities or endpoints exist yet.

## Reference

See the corresponding feature in `colab-life-test/` (read-only, git-ignored) for the
legacy behavior.
