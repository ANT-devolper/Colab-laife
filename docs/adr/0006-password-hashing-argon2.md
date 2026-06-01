# 0006. Password hashing with Argon2

- **Status:** Accepted
- **Date:** 2026-06-01

## Context

User login identities (`public.users`) store a `password_hash`. We need a password hashing
scheme that resists brute-force and GPU attacks and encodes its own parameters for future
rehashing.

## Decision

Hash passwords with **Argon2** in the `service` crate (`hash_password` / `verify_password`),
storing **PHC strings** that embed the algorithm, parameters and a per-password random salt.
A malformed stored hash is treated as a non-match rather than an error.

## Consequences

- Memory-hard hashing (Argon2) with per-password salts; equal passwords yield different
  hashes.
- Self-describing PHC strings allow parameters to evolve and support transparent rehashing
  later.
- Hashing logic lives in `service`, isolated from HTTP and the ORM, and is unit-tested
  without Docker.
- Argon2 is intentionally CPU/memory-intensive; parameters trade off login latency against
  attack resistance.
