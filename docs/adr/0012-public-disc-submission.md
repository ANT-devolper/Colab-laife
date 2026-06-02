# 0012. Public DISC submission (tenant resolved by schema, no auth)

- **Status:** Accepted
- **Date:** 2026-06-02

## Context

The DISC questionnaire (Phase 3B) is answered by a collaborator — and later a
recruitment candidate — who is **not** an authenticated user of the tenant. They
follow a link and submit their four dimension scores. The result must be written
into the right tenant schema, but the respondent has no session token, so the
`TenantContext` extractor ([ADR 0009](0009-per-request-tenant-resolution.md)),
which requires a `Bearer` JWT, cannot apply.

We must decide how an unauthenticated submission identifies its tenant and its
subject, and what protects the endpoint from abuse. Options considered:

- **Raw identifiers in the link (legacy approach).** The link carries the tenant
  `schema` and the subject's `collaborator_id` (a UUID); the submit endpoint is
  unauthenticated and trusts them, relying on the UUID's unguessability. Minimal
  moving parts; mirrors how the reference system worked.
- **Signed assessment token.** Mint a purpose-built token (signed with the
  existing `jwt_secret`) encoding `{ schema, subject, exp }`; the endpoint verifies
  the signature. Stateless, expirable, but more code and a new token type.
- **Persisted invitation token.** Store a random single-use token row per
  invitation. Revocable and auditable, but adds state and lifecycle management.

We want the questionnaire flow working end to end before investing in hardening,
and the existing `TenantRegistry` already resolves and validates a schema name
safely.

## Decision

Expose a **public, unauthenticated** `POST /public/disc-results` whose JSON body
carries the tenant `schema` plus the `collaborator_id` and the four scores
(matching the legacy raw-identifier approach).

- The tenant connection is resolved with `TenantRegistry::connection(&schema)`,
  which **validates the schema name** (`is_valid_schema_name`) before use; an
  invalid schema → `422`.
- The referenced collaborator must exist and be active, else `422`.
- No RBAC guard runs (the respondent is anonymous); security rests on the
  **unguessability of the `collaborator_id` UUID** carried in the link.
- The route bypasses the `TenantContext` extractor and reads `AppState` directly
  (the same shape as `POST /organizations`).

## Consequences

- The questionnaire can be answered without a login, and results land in the
  correct tenant schema — the flow works end to end.
- The protection is only the opaque UUID: anyone holding a valid `schema` +
  `collaborator_id` pair can post a result. There is no expiry, rate limit or
  revocation. This is acceptable for now (the data written is a self-reported
  questionnaire score) and is called out as a known trade-off.
- A future ADR can **supersede** this by introducing a signed or persisted
  assessment token without changing the stored data model — only the submission
  contract and the link format would change.
- The recruitment-candidate DISC result (Phase 3C) reuses this same decision for
  its own public submission endpoint.
