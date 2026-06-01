# 0001. Record architecture decisions

- **Status:** Accepted
- **Date:** 2026-06-01

## Context

This is a from-scratch reimplementation with full freedom to redesign architecture, API
contracts and data model. Decisions made early (language, framework, tenancy model) shape
everything downstream, and their rationale is easily lost over time.

## Decision

Keep lightweight Architecture Decision Records under `docs/adr/`, one file per decision,
numbered sequentially, using the template in [`README.md`](README.md). Record a new ADR
whenever a meaningful architectural decision is made. ADRs are immutable once Accepted; a
later ADR can supersede an earlier one.

## Consequences

- The *why* behind the design is preserved and reviewable.
- Revisiting a decision starts from its original context instead of guesswork.
- A small, ongoing cost: each significant decision must be written down (now enforced by
  the documentation directive in `CLAUDE.md`).
