# Dashboard

> **Status:** 🚧 planned — not implemented yet.

## Purpose

Give an organization an at-a-glance overview: aggregated metrics drawn from the other
modules (people, tasks, assessments, surveys, recruitment).

## Key concepts / entities

- **Metric / widget** — a single aggregated figure or chart (e.g. headcount, open tasks,
  survey participation).
- The dashboard mostly **reads** data owned by other modules rather than owning entities
  of its own.

## Main flows

- Load the organization overview.
- Drill from a metric into the underlying module.

## Permissions / roles

- Typically visible to admins/managers; the exact scoping is defined at design time.

## Status

Planned. Depends on the modules it aggregates.

## Reference

See the corresponding feature in `colab-life-test/` (read-only, git-ignored) for the
legacy behavior.
