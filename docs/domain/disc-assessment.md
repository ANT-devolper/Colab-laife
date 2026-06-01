# DISC assessment

> **Status:** 🚧 planned — not implemented yet.

## Purpose

Administer the DISC behavioral questionnaire to employees and produce a profile across the
four DISC dimensions: **D**ominance, **I**nfluence, **S**teadiness, **C**onscientiousness.

## Key concepts / entities

- **Questionnaire** — the set of DISC questions/items presented to a respondent.
- **Response** — an employee's answers to a questionnaire instance.
- **Profile / result** — the computed DISC scores derived from a response.

## Main flows

- Invite/assign an employee to take the assessment.
- Respondent completes the questionnaire.
- System scores the response and stores the resulting profile.
- View an employee's DISC profile.

## Permissions / roles

- Respondents complete their own assessment.
- Who may view results (the employee, their manager, admins) is defined at design time.

## Status

Planned. Scoring rules and the question bank are to be specified.

## Reference

See the corresponding feature in `colab-life-test/` (read-only, git-ignored) for the
legacy behavior and scoring.
