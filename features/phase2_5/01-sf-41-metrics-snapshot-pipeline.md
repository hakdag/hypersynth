# SF-41 — Metrics Snapshot Pipeline

## Purpose

Establish a daily aggregation pipeline that turns raw task, activity, and AI usage data into pre-computed KPI snapshots so that dashboards and trends can be served without scanning history on every request.

## Summary

This sub-feature introduces the `metric_snapshot` storage and a scheduled job that computes per-company, per-project, and per-user KPI values once per day. It is the data foundation that later Phase 2.5 sub-features read from. It produces value on its own because snapshots become queryable historical data even before any dashboard consumes them.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add the `metric_snapshot` entity (scope, metric_key, metric_value, payload, snapshot_date, scoping fields).
- Add a scheduled job that produces daily snapshots for every active company, project, and user.
- Compute an initial KPI set from data already available after Phase 2:
  - Throughput (tasks moved to Done per period)
  - On-time completion rate
  - Overdue rate
  - WIP per user (count of In Progress per assignee)
  - Unassigned task backlog
- Snapshots are immutable once written.
- Expose a minimal internal query API to read snapshots by scope and date range.

## Out of Scope

- Dashboards (covered by SF-42, SF-43, SF-44).
- Composite Project Health Score (SF-45).
- Insights and rule evaluation (SF-46, SF-47).
- AI ROI metrics (SF-50).
- Forecasting (SF-49).
- Export (SF-51).

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation
- SF-36 Activity Logs (used as a source for status transitions; if not yet deployed, throughput-style KPIs fall back to task timestamps where possible)

## Independent Deployment Notes

Ships independently. Snapshots accumulate immediately and become useful the moment any downstream sub-feature reads them. If no downstream consumer exists yet, the pipeline still produces auditable historical data.

## User Stories

- As a Company Admin, I want the platform to record daily KPI snapshots so that historical trends become possible.
- As a Project Manager, I want metric queries to be fast even as project history grows.

## Acceptance Criteria

- A daily snapshot job runs once per day per company and project.
- Each snapshot row contains scope, metric_key, metric_value, snapshot_date, and the relevant scoping IDs.
- Snapshots are not modified after creation.
- The initial KPI set listed in Scope is produced for every active project and company.
- Snapshot reads are scoped to the requester's company or personal account.

## Data Requirements

- New entity `metric_snapshot` with fields from Phase 2.5 FRD section 11.1.
- Either `company_id` or `owner_user_id` must be set.

## Security and Isolation Requirements

- Backend must enforce company / personal-account scoping on every read.
- A user must never read snapshots belonging to another company or personal account.
- Project-scoped snapshots must be readable only by project members and Company Admin.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
