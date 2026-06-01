# Documentation

This directory holds the project's in-depth documentation. It complements, and does not
duplicate, the top-level files (setup lives in `CONTRIBUTING.md`, process in `CLAUDE.md`).

## Map

| I want to understand… | Read |
|---|---|
| How the system fits together (crates, multi-tenancy, data flow) | [`architecture.md`](architecture.md) |
| A business module (notes, DISC, recruitment, …) | [`domain/`](domain/) |
| **Why** a technical decision was made | [`adr/`](adr/) |
| How to set up the project, pinned versions | [`../CONTRIBUTING.md`](../CONTRIBUTING.md) |
| The development process (TDD, commits, this doc rule) | [`../CLAUDE.md`](../CLAUDE.md) |
| What changed and when | [`../CHANGELOG.md`](../CHANGELOG.md) |

## Keeping docs honest

Documentation here is **mandatory to maintain** (see the "Documentation — mandatory"
section in [`../CLAUDE.md`](../CLAUDE.md)): a change that affects architecture, an API
contract, or a module's behavior updates the relevant page in the same task, before the
commit; a relevant architectural decision gets a new ADR. Pages mark **planned vs.
implemented** explicitly so the docs never overstate what the code does.
