# SF-45 — Project Health Score

## Purpose

Provide a single 0–100 composite indicator per project so that managers can scan many projects at a glance and identify those needing attention.

## Summary

This sub-feature introduces a composite Project Health Score computed from overdue, blocked, slip, and WIP penalties, each bounded and weighted using values stored in `health_score_config`. The score is exposed as an API value and rendered on the Project and Company dashboards when those are deployed. Weights are stored, never hard-coded, so they can be tuned later without code changes.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add `health_score_config` entity (per-company weights for overdue, blocked, slip, wip).
- Compute Project Health Score on each daily snapshot and store it as a `metric_snapshot` row.
- Expose the current score and its component breakdown through a read API.
- Render the score on the Project Dashboard (SF-42) and the at-risk list of the Company Dashboard (SF-43) when those exist.
- Use bounded penalties and clear formulas; document them in the API response.

## Out of Scope

- Tunable per-project weights (company-level only for now).
- Insight generation based on health score (handled by SF-46 / SF-47).
- Forecast-driven slip detection beyond the simple definition used here (SF-49 may refine).
- UI for editing weights (covered by SF-48).

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation
- SF-41 Metrics Snapshot Pipeline

## Independent Deployment Notes

Ships independently. If dashboards are not yet deployed, the score is still computed and queryable through the API. Default weights are used until SF-48 enables editing.

## User Stories

- As a Company Admin, I want a single number per project so I can quickly find projects that need attention.
- As a Project Manager, I want to see what is dragging the score down so I know where to focus.

## Acceptance Criteria

- A Project Health Score (0–100) is produced for every active project on each daily snapshot.
- The API returns both the score and its component breakdown (overdue, blocked, slip, wip penalties).
- Weights come from `health_score_config`; if no config exists, documented defaults are used.
- The score formula is deterministic and reproducible from the same inputs.

## Data Requirements

- New entity `health_score_config` with fields from Phase 2.5 FRD section 11.4.
- New `metric_key` values for the composite score and each component, stored as `metric_snapshot` rows.

## Security and Isolation Requirements

- Health score reads are scoped to the requester's company.
- Project-scoped scores are readable only by project members and Company Admin.
- Weight configuration is readable / writable only by Company Admin.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
