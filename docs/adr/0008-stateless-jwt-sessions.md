# 0008. Stateless authentication with JWT sessions

- **Status:** Accepted
- **Date:** 2026-06-01

## Context

The login identity lives in `public.users` (Argon2 `password_hash`, see
[ADR 0006](0006-password-hashing-argon2.md)). We need to turn a verified
email/password into something every subsequent request can present to prove who
the caller is and which tenant they belong to. Two broad options: **server-side
sessions** (a session store keyed by an opaque cookie/token) or **stateless
tokens** (signed claims the server can verify without a lookup).

In a schema-per-tenant design ([ADR 0004](0004-multi-tenant-schema-per-tenant.md))
the request must also carry enough to resolve the tenant schema. A session store
adds shared mutable state to operate, scale and invalidate; a signed token keeps
the API horizontally scalable with no session backend.

## Decision

Authenticate natively and issue **stateless JSON Web Tokens** (HS256, signed with
a server-held secret). On `POST /auth/login`, `service::auth::authenticate`
verifies the credentials against `public` (unknown email, soft-deleted user and
wrong password all collapse to one `InvalidCredentials` outcome to prevent user
enumeration; a deactivated organization is `OrganizationInactive`). On success we
encode `Claims { sub, org, schema, is_admin, exp }` and return the token.

- **HS256** (symmetric HMAC) with a single secret read from `JWT_SECRET`. The
  pure-Rust `rust_crypto` backend of `jsonwebtoken` v10 provides it — no C
  toolchain. Asymmetric keys (RS/ES) are deferred until multiple verifying
  parties exist.
- **`schema` travels in the claims** so the planned auth middleware resolves the
  tenant from the verified token with no extra database round-trip.
- **Expiry only** (`DEFAULT_TTL`, 24h). There is no server-side session state and
  therefore no revocation list yet.

## Consequences

- The API stays stateless: any instance verifies a token with the shared secret;
  no session store to run or scale.
- The tenant schema is available from the token alone, which the upcoming
  middleware (and per-request schema resolution) builds on.
- Tokens cannot be revoked before they expire; the 24h TTL bounds the exposure.
  Refresh tokens and revocation are deferred until needed.
- The `JWT_SECRET` is now required configuration; leaking or rotating it
  invalidates or forges sessions, so it must be a strong, protected secret in
  every non-dev environment.
- Symmetric signing means every service that verifies tokens shares the secret;
  moving to asymmetric keys would need a new ADR.
