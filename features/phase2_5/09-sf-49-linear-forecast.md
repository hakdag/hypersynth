# SF-49 — Linear Forecast and Velocity Trends

## Purpose

Project an expected completion date per project based on current throughput so managers can see slip before deadlines hit.

## Summary

This sub-feature computes a simple linear forecast for each active project: remaining open tasks divided by rolling throughput gives an expected completion date. It also stores velocity trend slope so the Forecast slip insight (SF-47) can detect when the projection moved later than the previous snapshot. The forecast is deterministic — no machine learning — and intentionally simple. Phase 3 may layer richer, confidence-scored forecasts on top.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Compute, on each daily snapshot, for every active project:
  - Rolling 4-week throughput
  - Velocity trend slope (rolling throughput over time)
  - Projected completion date = today + (open_tasks / rolling_throughput_per_day)
- Store these as `metric_snapshot` rows.
- Expose the forecast on the Project Dashboard (SF-42) when it is deployed.
- Make the previous-snapshot forecast value available for the Forecast slip insight rule (SF-47).

## Out of Scope

- Confidence intervals, Monte Carlo, or any probabilistic model.
- Per-feature forecasting (only project-level here).
- Forecast for projects with no completed tasks (returns "insufficient data").
- AI-generated forecasts (Phase 3).

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation
- SF-41 Metrics Snapshot Pipeline

## Independent Deployment Notes

Ships independently. If SF-42 is not yet deployed, the forecast is queryable through the API. If SF-47 is not yet deployed, the slip insight simply does not fire — the forecast still exists.

## User Stories

- As a Project Manager, I want to see when a project is projected to complete so I can plan.
- As a Company Admin, I want to know when projections have slipped compared to the previous snapshot.

## Acceptance Criteria

- A projected completion date is computed for every active project that has enough throughput data.
- Projects without sufficient data return a documented "insufficient data" state instead of an error.
- The previous snapshot's forecast is retrievable so slip detection can compare values.
- Computation is deterministic and reproducible from the same snapshot inputs.

## Data Requirements

- No new entities. New `metric_key` values stored as `metric_snapshot` rows.

## Security and Isolation Requirements

- Forecast reads are scoped to the requester's company.
- Project-scoped forecasts are readable only by project members and Company Admin.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
