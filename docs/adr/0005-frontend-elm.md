# 0005. Frontend in Elm

- **Status:** Accepted
- **Date:** 2026-06-01

## Context

The product needs a web frontend that talks to the backend over HTTP/JSON. We want a UI
stack with strong correctness guarantees that matches the project's reliability-first
posture.

## Decision

Build the frontend in **Elm**, following The Elm Architecture, communicating with the
backend over HTTP/JSON via decoders/encoders. Unit tests use `elm-test`; full-stack flows
are covered by Playwright E2E.

## Consequences

- No runtime exceptions in practice; the compiler enforces exhaustive handling and a clear
  update/view/model structure.
- Explicit JSON decoders/encoders make the API contract boundary visible and testable.
- Smaller talent pool and ecosystem than mainstream JS frameworks; some interop friction
  via ports when wrapping JS libraries.
