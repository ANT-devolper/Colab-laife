# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working in this
repository.

## Project

This repository is a **from-scratch reimplementation of ColabLife** — a multi-tenant
enterprise SaaS (notes/feedback, DISC assessment, dashboard, employees, tasks,
recruitment, whistleblower channel, climate survey).

The reference implementation lives in `colab-life-test/`. It is **reference-only**:
read it to understand behavior, but **never commit it** (it is git-ignored). We have
**full freedom to redesign** architecture, API contracts and data model — the old system
is a functional reference, not a blueprint.

### Pair programming (Extreme Programming)

We work as an XP pair:

- **Claude (driver):** writes production code and tests, runs the suite.
- **User (navigator):** reviews, makes requests, and guards quality.

### Project language

**Everything in the project is written in English** — code, comments, identifiers,
documentation, `CLAUDE.md`, `CHANGELOG.md`, and commit messages. (Pair-programming
conversation with the user happens in Portuguese; the artifacts do not.)

## ⚠️ Mandatory directive — TDD first

**No implementation or refactor may be done without tests that cover it.** This rule
overrides any request to the contrary:

1. **Test before code.** Before creating or changing behavior, write the tests that
   describe the expected behavior first.
2. **Red → Green → Refactor.** Run the tests and confirm they fail (red); implement the
   minimum to pass (green); refactor while keeping green.
3. **No regression.** Every change ends with the full suite 100% green. Never commit
   with a failing test.
4. **Bug → regression test.** Every bug found gets a failing test that reproduces it
   first, then the fix. The test prevents the bug from coming back in regressions.
5. **Refactor only with a safety net.** Do not refactor a file without tests locking the
   current behavior; if they don't exist, write characterization tests first.
6. **No tests provided → ask.** If the user requests an implementation without specifying
   the tests, **ask which test cases / expected behavior** are required before writing any
   code. Never assume and proceed straight to implementation.

## Engineering practices (XP)

- Small, focused steps.
- Clear names; no dead code.
- Strict comparisons; no unnecessary mutation.
- Comments explain the **why**, not the **what**.

## CHANGELOG — mandatory

**Every meaningful point of the implementation must be recorded in `CHANGELOG.md`**
(Keep a Changelog format), in the same task as the change, **before the commit**.

## Commit messages — Conventional Commits

- Format: `type(scope): description` (scope optional).
- Types: `feat, fix, refactor, perf, docs, test, style, chore, build, ci`.
- Subject in the imperative mood, lowercase, up to ~50 characters, no trailing period.
- Each commit must be **atomic** (a single logical change).
- If a body is present: blank line after the subject, explain the **why** (not the how),
  wrapped at ~72 columns.
- Breaking change: use `type!:` in the subject, or a `BREAKING CHANGE: ...` footer.
- Reference issues in the footer when applicable (e.g. `Refs #123`).

Example: `feat(auth): add refresh token via cookie`

## Commit flow

1. Run the full test suite → ensure it is green.
2. Update `CHANGELOG.md`.
3. Commit (Conventional Commits, in English).

Work is done **directly on `main`** for now (no feature branches). Claude **commits
automatically** once the flow above is satisfied. **Pushing is manual — only the user
pushes.** Claude never pushes.

## Stack — TO BE DEFINED

- **Backend: Rust** (fixed).
- **Web framework:** TBD.
- **Persistence / database:** TBD.
- **Frontend:** TBD.
- **Deploy target** (serverless vs. server/container): TBD.

These will be filled in here as we decide them.

## Essential commands — TO BE DEFINED

To be filled in once the Rust scaffold exists (e.g. `cargo test`, `cargo build`).
