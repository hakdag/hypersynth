# SF-42 — Project Dashboard Insights Extension

## Purpose

Extend the existing Project Dashboard (SF-39) with the Phase 2.5 KPIs so that project managers can see flow, bottlenecks, and trends in one place.

## Summary

This sub-feature adds new widgets to the Project Dashboard: cycle time and lead time distributions, time-in-status, blocked-task aging, dependency bottleneck list, throughput / on-time / overdue trends, and a burndown / burnup chart. It reads from `metric_snapshot` where available and falls back to live queries for current-day values. Widgets gracefully omit themselves when their input data is not yet available.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add widgets to the existing per-project dashboard:
  - Cycle time (median) and lead time (median)
  - Time-in-status breakdown
  - Blocked-task aging list (p90 + raw list)
  - Reopen rate
  - Throughput, on-time rate, and overdue rate trend lines (7 / 30 / 90 day windows)
  - Burndown / burnup chart
  - Dependency bottleneck list (tasks blocking the most others)
- Every KPI must show its formula in a tooltip.
- Dashboards prefer snapshot reads; live queries are used only for the current day.

## Out of Scope

- Composite Project Health Score (SF-45).
- Forecast / projected completion date (SF-49).
- AI ROI section (SF-50).
- Company-wide and personal dashboards (SF-43, SF-44).
- Export (SF-51).

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation
- SF-17 Project Membership
- SF-39 Project Dashboard
- SF-41 Metrics Snapshot Pipeline (preferred source; widgets fall back to live data if a snapshot is missing)

## Independent Deployment Notes

Ships independently of other Phase 2.5 sub-features. If SF-45 (health score), SF-49 (forecast), or SF-50 (AI ROI) are not yet deployed, their sections are simply not rendered.

## User Stories

- As a Project Manager, I want to see cycle time and blocked-task aging so that I can spot flow problems early.
- As a Project Manager, I want trends over 7 / 30 / 90 days so that I can tell if the project is improving.
- As a Contributor, I want to see what is blocking progress on my project.

## Acceptance Criteria

- Each new widget displays the documented KPI with its formula visible to the user.
- Trend windows of 7, 30, and 90 days are supported.
- Widgets without required input data render an empty state rather than failing.
- Dashboard access remains restricted to project members and authorized roles.
- KPI values for past days are read from `metric_snapshot`; current-day values are computed live.

## Data Requirements

- No new persistent entities.
- Reads from existing Task, Activity Log, Dependency, and `metric_snapshot` (SF-41).

## Security and Isolation Requirements

- Dashboard data is scoped to the requested project and the requester's company.
- Backend authorization is enforced on every read.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
