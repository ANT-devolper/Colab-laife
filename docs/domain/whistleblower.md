# Whistleblower channel

> **Status:** 🚧 planned — not implemented yet.

## Purpose

Provide a confidential reporting channel so employees can raise concerns (misconduct,
harassment, compliance) with appropriate confidentiality.

## Key concepts / entities

- **Report** — a submitted concern, possibly **anonymous**, with a status as it is
  triaged.
- **Handler** — the restricted role that reviews and responds to reports.

## Main flows

- Submit a report (optionally anonymously).
- Triage and update a report's status.
- Communicate back to the reporter without breaking confidentiality.

## Permissions / roles

- Strongly restricted: only designated handlers may read reports. Confidentiality and (for
  anonymous reports) non-identifiability are first-class requirements, defined carefully at
  design time.

## Status

Planned. Sensitive — its data handling and access control are specified before
implementation.

## Reference

See the corresponding feature in `colab-life-test/` (read-only, git-ignored) for the
legacy behavior.
