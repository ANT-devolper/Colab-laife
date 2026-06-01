# 0002. Backend in Rust with Axum

- **Status:** Accepted
- **Date:** 2026-06-01

## Context

We need a backend for a multi-tenant SaaS that is reliable under concurrency and can run
both as a long-running server and, if needed, on AWS Lambda. The reference system is a
functional guide only — we are free to choose the stack.

## Decision

Implement the backend in **Rust** using the **Axum** web framework (Tokio/Tower ecosystem).
Axum runs as a long-running server or on Lambda, sharing the same handler code.

## Consequences

- Strong compile-time guarantees and performance; fewer whole classes of runtime bugs.
- Rich async ecosystem (Tokio/Tower) for middleware and integration testing (`axum-test`).
- Deployment flexibility: container/server now, Lambda possible later.
- Steeper learning curve and longer compile times than a dynamic-language backend.
- See [0003](0003-persistence-seaorm-postgres.md) for persistence within this stack.
