# SF-24 — Audit Logging

## Purpose

Record critical actions across the platform so that company admins and the System Admin can review what happened, by whom, and when.

## Summary

This sub-feature introduces the Audit Log entity and the recording mechanism. It captures company-scoped events (registration, profile changes, invitations, role changes, project/feature/task/document changes, AI settings changes, AI operations) and System Admin events (login attempts, company status changes, user status changes, configuration changes).

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add the Audit Log entity with all required fields.
- Provide a unified API for recording an audit event used by all relevant features.
- Record the company audit events listed in the Phase 1 FRD section 10.1.
- Record the System Admin audit events listed in the Phase 1 FRD section 10.2.
- Capture context fields: ip_address, user_agent, metadata (action-specific JSON).

## Out of Scope

- The user-facing or admin-facing audit log viewer (covered by SF-26 for the System Admin viewer).
- Long-term archival, export, or retention policies beyond simple persistence.
- Real-time alerting on suspicious actions.

## Dependencies

- SF-14 Company Data Isolation
- SF-13 User Roles and Authorization

## Independent Deployment Notes

Can be deployed independently. As soon as it ships, all instrumented features start producing audit records, even before any viewer exists. Operators can still query the underlying store directly for value.

## User Stories

- As a Company Admin, I want sensitive actions on my company recorded so that there is accountability.
- As a System Admin, I want platform-level actions recorded so that I can investigate incidents.

## Acceptance Criteria

- Each listed company event produces an audit log record with correct company_id, user_id, action_type, entity_type/id, and metadata.
- Each listed System Admin event produces an audit log record with system_admin_email set and no company_id (unless the event is about a specific company).
- ip_address and user_agent are recorded when available.
- Audit records are immutable: the system provides no API to edit or delete them.
- Recording an audit event must not block or break the originating action; failures to write an audit record must be logged but must not roll back the action.

## Data Requirements

- Audit Log: id, company_id nullable, user_id nullable, system_admin_email nullable, action_type, entity_type, entity_id nullable, metadata, ip_address nullable, user_agent nullable, created_at.

## Security and Isolation Requirements

- Audit records inherit company isolation (SF-14): a company user must only see audit logs of their own company when viewers are exposed.
- Audit records must not store secrets such as passwords or API keys.
- Audit log access for the System Admin is controlled by SF-26.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
