# SF-50 — AI ROI Metrics

## Purpose

Surface deterministic AI return-on-investment KPIs so company management can decide whether AI usage is paying off.

## Summary

This sub-feature computes AI-specific KPIs from the Phase 1 AI usage records: AI utilization (operations per active user), AI cost per shipped task (sum of estimated cost divided by tasks moved to Done in the period), AI task survival rate (AI-created tasks that reach Done vs. all AI-created tasks), and AI error rate (failed operations vs. total). Results are stored as `metric_snapshot` rows and rendered on the Company and Project dashboards where applicable.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Compute the following KPIs on each daily snapshot:
  - AI utilization
  - AI cost per shipped task
  - AI task survival rate
  - AI error rate
- Compute at company, project, and (where applicable) user scope.
- Store as `metric_snapshot` rows.
- Render in dedicated AI ROI sections on the Company Dashboard (SF-43) and Project Dashboard (SF-42) when those are deployed.
- Provide the input the AI failure spike rule (SF-47) requires.

## Out of Scope

- Cost predictions or budgeting.
- Cross-company AI benchmarking (Phase 4).
- AI vendor comparison reports.
- AI proposal acceptance rate metrics (those arrive with Phase 3).

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation
- SF-23 AI Usage Tracking
- SF-41 Metrics Snapshot Pipeline

## Independent Deployment Notes

Ships independently. If no AI usage has occurred, KPIs are zero and the dashboards render an empty state. The AI failure spike rule (SF-47) becomes meaningful once this sub-feature is deployed.

## User Stories

- As a Company Admin, I want to know how much AI is costing us per shipped unit of work.
- As a Company Admin, I want to see whether AI-created tasks actually get done, or are mostly deleted.
- As a Project Manager, I want to know whether AI usage on my project is failing more than usual.

## Acceptance Criteria

- All four KPIs are computed and stored on each daily snapshot.
- Formulas are documented in the API response and the dashboard tooltip.
- KPIs return zero or a documented "no data" state when AI usage is absent.
- KPI values are scoped to the requester's company.

## Data Requirements

- No new entities. New `metric_key` values stored as `metric_snapshot` rows.
- Reads from existing AI usage records (SF-23) and Task data.

## Security and Isolation Requirements

- AI ROI reads are scoped to the requester's company.
- Project-scoped values are readable only by project members and Company Admin.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
