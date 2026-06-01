# Architecture Decision Records

An **ADR** captures a single architectural decision: its context, the decision itself, and
its consequences. We record them so the *why* behind the design survives, and so revisiting
a choice later starts from the original reasoning.

When a meaningful architectural decision is made, add the next-numbered ADR (see the
documentation directive in [`../../CLAUDE.md`](../../CLAUDE.md)). ADRs are immutable once
**Accepted**; to change a decision, add a new ADR that **supersedes** the old one and
update the old one's status to *Superseded by NNNN*.

## Index

| # | Title | Status |
|---|---|---|
| [0001](0001-record-architecture-decisions.md) | Record architecture decisions | Accepted |
| [0002](0002-backend-rust-axum.md) | Backend in Rust with Axum | Accepted |
| [0003](0003-persistence-seaorm-postgres.md) | Persistence with SeaORM over PostgreSQL | Accepted |
| [0004](0004-multi-tenant-schema-per-tenant.md) | Multi-tenancy: schema per tenant | Accepted |
| [0005](0005-frontend-elm.md) | Frontend in Elm | Accepted |
| [0006](0006-password-hashing-argon2.md) | Password hashing with Argon2 | Accepted |
| [0007](0007-tenant-provisioning.md) | Tenant provisioning and schema resolution | Accepted |
| [0008](0008-stateless-jwt-sessions.md) | Stateless authentication with JWT sessions | Accepted |
| [0009](0009-per-request-tenant-resolution.md) | Per-request tenant schema resolution and auth extractor | Accepted |

## Template

```markdown
# NNNN. <Title>

- **Status:** Proposed | Accepted | Superseded by NNNN
- **Date:** YYYY-MM-DD

## Context

What forces are at play — the problem, constraints and options considered.

## Decision

The choice we made, stated plainly.

## Consequences

What becomes easier or harder as a result (positive and negative).
```
