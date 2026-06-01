# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Project guidelines and development process (XP pair programming, TDD with
  Red → Green → Refactor, no-regression rule, regression test for every bug, mandatory
  CHANGELOG, Conventional Commits) in `CLAUDE.md`.
- Commit policy: Claude commits automatically; pushing is manual (user only).
- Project stack definition: Rust + Axum + SeaORM over PostgreSQL (backend), Elm
  (frontend); deploy target and mobile left TBD.
- Testing stack definition: cargo test + rstest + mockall (backend unit), axum-test +
  testcontainers (backend integration), elm-test (frontend unit), Playwright (E2E).
- `.gitignore` ignoring the reference system `colab-life-test/` and Rust/editor artifacts.
- This `CHANGELOG.md`.
