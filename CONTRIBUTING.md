# Contributing

This is a from-scratch reimplementation of ColabLife. Please read `CLAUDE.md` first — it
defines the mandatory process (XP pair programming, TDD Red→Green→Refactor, full suite
green before every commit, a regression test for every bug, Conventional Commits, and
English for all project artifacts).

## Repository layout

```
backend/      Rust Cargo workspace
  crates/
    api/        Axum HTTP application (+ tests/ for integration tests)
    entity/     SeaORM entities
    migration/  sea-orm-migration (run via `cargo run -p migration`)
    service/    business logic / domain services
frontend/     Elm application (elm-test for unit tests)
e2e/          Playwright end-to-end tests
docker-compose.yml   dev PostgreSQL
justfile      task runner (setup / test / run / migrate / fmt / db-up / db-down)
```

## Pinned versions

Versions are pinned for reproducibility. Lockfiles (`backend/Cargo.lock`,
`*/package-lock.json`) are committed and are the source of truth for exact (including
transitive) versions; the manifests below pin the direct ones.

| Tool / dependency | Version | Pinned in |
|---|---|---|
| Rust toolchain | 1.95.0 | `rust-toolchain.toml` |
| Node.js | 26.1.0 | `frontend/.nvmrc`, `e2e/.nvmrc` |
| Elm (compiler) | 0.19.1 (npm `elm` 0.19.1-6) | `frontend/package.json` |
| elm-test | 0.19.1-revision17 | `frontend/package.json` |
| elm-format | 0.8.8 | `frontend/package.json` |
| Playwright | 1.60.0 | `e2e/package.json` |
| PostgreSQL | 16.14 | `docker-compose.yml` (match in integration tests) |
| axum | 0.8.9 | `backend/Cargo.toml` |
| tokio | 1.52.3 | `backend/Cargo.toml` |
| sea-orm / sea-orm-migration | 1.1.20 | `backend/Cargo.toml` |
| serde | 1.0.228 | `backend/Cargo.toml` |
| axum-test | 20.1.0 | `backend/Cargo.toml` |
| testcontainers / -modules | 0.27.3 / 0.15.0 | `backend/Cargo.toml` |
| rstest | 0.26.1 | `backend/Cargo.toml` |
| mockall | 0.14.0 | `backend/Cargo.toml` |
| pretty_assertions | 1.4.1 | `backend/Cargo.toml` |

## Prerequisites (install once)

- **Rust** — `rustup` is recommended so the toolchain in `rust-toolchain.toml` (1.95.0) is
  honored automatically. A system Rust ≥ 1.95 also works.
- **Node.js ≥ 22** — `nvm` is recommended; run `nvm use` in `frontend/` and `e2e/` to pick
  up the `.nvmrc`.
- **just** — the task runner. Install with `cargo install just`, or `pacman -S just` /
  `brew install just`.
- **Docker** — required for `just db-up` (dev PostgreSQL) and the backend integration
  tests (testcontainers). On WSL, enable Docker Desktop's WSL integration, or install a
  Docker engine inside the distro.
- **Playwright host libraries** — the browser binaries are downloaded by `just setup`, but
  running them needs system libraries: `sudo npx playwright install-deps` (Debian/Ubuntu)
  or the equivalent packages on your distro.

## Getting started

```bash
git clone <repo> && cd Colab-laife
just setup     # builds backend, installs frontend & e2e deps, downloads browsers
just test      # backend (cargo test) + frontend (elm-test) + e2e (playwright)
just run       # start the backend API
```

Database (when working on persistence):

```bash
just db-up                 # start PostgreSQL 16.14 in Docker
just migrate up            # apply migrations (uses DATABASE_URL)
just db-down               # stop it
```

`DATABASE_URL` defaults to `postgres://colab:colab@localhost:5432/colab_life` (see the
`justfile`); override it via the environment when needed.

## Optional tools

- **sea-orm-cli** (only for generating entities from an existing database):
  `cargo install sea-orm-cli --version 1.1.20`. Day-to-day migrations do **not** need it —
  use `just migrate ...` (the `migration` crate's binary).
