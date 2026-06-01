# 0011. Serve the Elm SPA from the Axum binary (single origin)

- **Status:** Accepted
- **Date:** 2026-06-01

## Context

Phase 2 introduces the Elm frontend ([ADR 0005](0005-frontend-elm.md)), which
talks to the Axum backend over HTTP/JSON. We must decide how the browser obtains
the built SPA and how it reaches the API. Two broad topologies:

- **Separate origins** — a static host/CDN serves the SPA on one origin, the API
  on another. The browser then makes cross-origin requests, so the API needs CORS
  configuration, and local development and the end-to-end test must orchestrate
  two servers.
- **Single origin** — the Axum binary serves both the API routes and the built
  SPA's static files. Same-origin requests need no CORS, and one process covers
  local dev and the Playwright E2E (which boots the real stack).

The app is a single deployable unit for now ([ADR 0002](0002-backend-rust-axum.md)
keeps the option of a long-running server), and the foundation already routes
every authenticated request through the backend. We value operational simplicity
over independent frontend/backend scaling at this stage.

## Decision

Serve the built Elm SPA **from the Axum binary, on the same origin as the API**.

- A `with_static_spa(router, dist_dir)` combinator (`api` crate) adds a
  `tower-http` `ServeDir` as the router's `fallback_service`, with a
  `ServeFile` of `index.html` as the directory's own fallback. Real files (the
  compiled `app.js`, assets) are served from disk; any other path returns
  `index.html` so the SPA boots and routes on the client.
- **API routes keep precedence**: they are matched first; the static fallback
  runs only for unmatched paths. No API path can be shadowed by a static file.
- The SPA is **optional at runtime**: `main` wraps the router only when the
  `FRONTEND_DIST` environment variable is set, so the same binary can also run
  API-only. The `justfile` builds the SPA (`frontend-build`: `elm make` into
  `frontend/dist`, plus the HTML shell) and points `FRONTEND_DIST` at it.
- New workspace dependency: `tower-http` with the `fs` feature.

## Consequences

- No CORS layer to configure or keep in sync; the browser only ever talks to one
  origin.
- Local development and the Playwright E2E run a single process serving both the
  SPA and the API — simpler orchestration.
- The frontend is not independently scalable or CDN-hosted; if that becomes
  necessary, a future ADR can move to separate origins (and add CORS) without
  changing the API contracts.
- The backend depends on a built `frontend/dist`; when `FRONTEND_DIST` is unset
  the server is API-only (the default in pure-API test setups), so existing
  integration tests are unaffected.
- Static serving never touches the database, so it is covered by a fast
  integration test using a `MockDatabase` (no Docker).
