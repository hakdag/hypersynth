# SF-26 — System Admin Dashboard: Audit Log Viewer

## Purpose

Allow the System Admin to view and filter audit logs across the platform.

## Summary

This sub-feature provides an audit log viewer in the System Admin dashboard. The admin can browse recent events and filter by company, user, action type, and date range.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Provide an audit log viewer page in the System Admin dashboard.
- Show audit records ordered by time, most recent first.
- Filter by company, user, action_type, and date range.
- Show entity_type, entity_id, metadata, ip_address, and user_agent fields per record.
- Paginate large result sets.

## Out of Scope

- Editing or deleting audit records.
- Exporting audit logs.
- Real-time tailing.
- Company-facing audit views.

## Dependencies

- SF-18 System Admin Authentication
- SF-24 Audit Logging

## Independent Deployment Notes

Can be deployed any time after audit logging exists. Without SF-24, this sub-feature has no data to show and should not be enabled.

## User Stories

- As a System Admin, I want to filter audit events to investigate a specific incident.
- As a System Admin, I want to confirm that critical actions were captured.

## Acceptance Criteria

- The System Admin can list audit logs with the most recent first.
- Filters work for company, user, action_type, and date range.
- Pagination works for large data sets.
- Audit records remain read-only.
- Only System Admin sessions can access this viewer.

## Data Requirements

- Reuses the Audit Log entity from SF-24.

## Security and Isolation Requirements

- Only System Admin sessions can access the viewer.
- Records must not be modifiable through this screen.
- The viewer must not display secrets even if metadata mistakenly contains them; sensitive keys should never be present per SF-24.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
