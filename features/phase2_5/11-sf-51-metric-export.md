# SF-51 — KPI Export (CSV / JSON)

## Purpose

Allow users to export any KPI table they can see on a dashboard as CSV or JSON for offline analysis or sharing.

## Summary

This sub-feature adds export endpoints that serve the same KPI data the dashboards render, formatted as CSV or JSON. Export requests reapply the same authorization checks as the dashboard reads, so a user can only export what they can already see. No scheduled exports, no email delivery, no external storage push — only direct in-app download.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add export endpoints for KPI tables in CSV and JSON.
- Cover the KPI surfaces produced by SF-41 / SF-42 / SF-43 / SF-44 / SF-45 / SF-49 / SF-50 (whichever are deployed).
- Apply the same scoping rules used by the dashboards on every export request.
- Stream large exports rather than buffering whole result sets in memory.

## Out of Scope

- Scheduled / recurring exports.
- Email delivery of exports.
- Export to external storage (S3 etc.).
- Export of raw audit logs or activity logs.
- PDF or spreadsheet (XLSX) formats.

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation
- SF-41 Metrics Snapshot Pipeline
- At least one dashboard sub-feature (SF-42, SF-43, or SF-44) to give the export meaningful surface coverage

## Independent Deployment Notes

Ships independently. As more dashboard sub-features come online, the export coverage automatically grows because exports reuse the same underlying queries.

## User Stories

- As a Company Admin, I want to export KPI tables so I can share them with leadership.
- As a Project Manager, I want to download my project metrics as CSV so I can analyse them offline.

## Acceptance Criteria

- A user can download any KPI table they can view on a dashboard as CSV or JSON.
- Export requests are rejected if the user cannot view the underlying data.
- Exported values match the dashboard values for the same scope and period.
- Large exports do not exhaust server memory.

## Data Requirements

- No new persistent entities.
- Reads from `metric_snapshot` (SF-41) and existing live query paths used by the dashboards.

## Security and Isolation Requirements

- Authorization is re-checked at export time, not delegated to the frontend.
- A user can never export another company's or another personal account's data.
- Project-scoped exports are restricted to project members and Company Admin.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
