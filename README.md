# Colab-laife

A from-scratch reimplementation of **ColabLife** — a multi-tenant enterprise SaaS for
people operations: notes/feedback, DISC assessment, dashboard, employees, tasks,
recruitment, a whistleblower channel and a climate survey.

## Status

**Early development — foundation complete, first vertical slice in progress.** What exists
today:

- HTTP liveness/readiness health probes (Axum).
- Multi-tenant foundation: `organizations`/`users` in the cross-tenant `public` schema, tenant
  provisioning (a dedicated schema per organization), Argon2 password hashing, stateless JWT
  login, a per-request tenant resolver + auth extractor, and granular per-tenant RBAC.
- Cadastro backend in the tenant schema: `sector`, `role` and `collaborator`, each with
  RBAC-guarded CRUD.
- People domains (in progress): the notes & feedback backend — `feedback` with its
  expectation-contract items and scored behaviors, plus standalone `annotation` notes — each
  with RBAC-guarded CRUD.
- Elm SPA: a login page that obtains a session token and a read-only directory
  (collaborators/sectors/roles), served from the Axum binary on the same origin. Covered by a
  first Playwright end-to-end test.

The eight business modules above are otherwise **planned** — see [`docs/domain/`](docs/domain/)
for their intended scope.

## Stack

- **Backend:** Rust + [Axum](https://github.com/tokio-rs/axum), persistence with
  [SeaORM](https://www.sea-ql.org/SeaORM/) over PostgreSQL.
- **Frontend:** [Elm](https://elm-lang.org/) (HTTP/JSON).
- **Deploy target / mobile:** TBD.

## Quickstart

```bash
git clone <repo> && cd Colab-laife
just setup     # build backend, install frontend & e2e deps, download browsers
just test      # backend (cargo test) + frontend (elm-test) + e2e (playwright)
just run       # start the backend API
```

Prerequisites, pinned versions and database commands live in
[`CONTRIBUTING.md`](CONTRIBUTING.md).

## Documentation

- [`docs/`](docs/) — documentation map (start here).
- [`docs/architecture.md`](docs/architecture.md) — system overview, crates, multi-tenancy.
- [`docs/domain/`](docs/domain/) — the business modules and glossary.
- [`docs/adr/`](docs/adr/) — Architecture Decision Records (why we chose what we chose).

## Contributing

Read [`CONTRIBUTING.md`](CONTRIBUTING.md) for setup and [`CLAUDE.md`](CLAUDE.md) for the
mandatory process (XP pair programming, TDD Red→Green→Refactor, full suite green before
every commit, mandatory CHANGELOG and documentation updates, Conventional Commits).
Changes are recorded in [`CHANGELOG.md`](CHANGELOG.md).
