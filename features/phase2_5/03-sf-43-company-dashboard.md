# SF-43 — Company Dashboard

## Purpose

Give Company Admins a portfolio-level view of project health, throughput, workload, and AI usage across the entire company.

## Summary

This sub-feature introduces a new top-level Company Dashboard. It aggregates company-wide KPIs from `metric_snapshot` and surfaces project count by status, top at-risk projects, a company-wide throughput trend, a workload heatmap across users, and a dependency bottleneck list spanning projects. AI ROI and Project Health Score sections are rendered when their respective sub-features are deployed.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add a new Company Dashboard view accessible to Company Admin (full) and Project Manager (read).
- Sections:
  - Portfolio overview (projects grouped by status)
  - Top at-risk projects list
  - Company-wide throughput trend
  - Workload heatmap (open task counts per user)
  - Top dependency bottlenecks across all projects
- Sections that depend on sub-features not yet deployed (e.g. Project Health Score, AI ROI) are hidden gracefully.

## Out of Scope

- Project-level and personal dashboards (SF-42, SF-44).
- Composite health score computation (SF-45).
- Insights panel (SF-46, SF-47).
- AI ROI rendering (SF-50).
- Forecast (SF-49).
- Export (SF-51).
- Custom or saved dashboard layouts.

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation
- SF-41 Metrics Snapshot Pipeline

## Independent Deployment Notes

Ships independently. Without SF-45, the "at-risk" list falls back to a simple overdue-and-blocked heuristic. Without SF-50, the AI ROI section is omitted.

## User Stories

- As a Company Admin, I want a single screen showing all projects and which ones need attention.
- As a Company Admin, I want to see how workload is distributed across people so I can rebalance.
- As a Project Manager, I want a read-only view of the wider portfolio.

## Acceptance Criteria

- The Company Dashboard is accessible to Company Admin (full) and Project Manager (read).
- All listed sections render with data scoped to the requester's company.
- Sections gracefully omit themselves when their input sub-features are not deployed.
- Users from other companies cannot access the dashboard data.

## Data Requirements

- No new persistent entities.
- Reads from `metric_snapshot` (SF-41) and existing Project, Task, and Membership data.

## Security and Isolation Requirements

- Dashboard data is strictly scoped to the requester's company.
- Backend authorization enforces role gating; frontend hiding is not sufficient.
- Personal Accounts do not see a Company Dashboard.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
