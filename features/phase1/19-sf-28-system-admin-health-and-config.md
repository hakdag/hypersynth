# SF-28 — System Admin Dashboard: System Health and Configuration

## Purpose

Allow the System Admin to view platform health indicators and manage global configuration values.

## Summary

This sub-feature provides two related areas inside the System Admin dashboard: a System Health overview that shows application, background job, AI provider, email delivery, and storage status; and a Configuration area where the System Admin can manage allowed AI providers, global usage limits, a platform announcement, and feature flags.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Provide a System Health page showing: application status, background job status, AI provider error rate, email delivery status, storage usage.
- Provide a Configuration page allowing the System Admin to manage:
  - Allowed AI providers
  - Global usage limits
  - Platform announcement message
  - Feature flags
- Persist configuration changes in a dedicated configuration store.
- Apply configuration values across the platform (e.g., feature flags affect feature visibility; allowed AI providers gate the company AI settings choices).
- Record configuration changes in audit logs when audit logging is available.

## Out of Scope

- Detailed metrics dashboards (charts, time-series graphs).
- External alerting integrations.
- Runtime hot-reload of code.

## Dependencies

- SF-18 System Admin Authentication

## Independent Deployment Notes

Can be deployed in stages. The System Health page can ship first using simple status checks; the Configuration page can ship next, with each configuration value (allowed AI providers, limits, announcement, feature flags) introduced one at a time. Each piece adds value on its own.

## User Stories

- As a System Admin, I want to see the platform's health at a glance so that I can react quickly to outages.
- As a System Admin, I want to control allowed AI providers and feature flags so that I can govern the platform without code changes.

## Acceptance Criteria

- The System Health page shows the documented status indicators.
- The Configuration page allows editing of allowed AI providers, global usage limits, platform announcement, and feature flags.
- Configuration changes are persisted and applied to subsequent platform behavior.
- All configuration changes are recorded in audit logs when audit logging is available.
- Only System Admin sessions can access these pages.

## Data Requirements

- A configuration store (entity or equivalent) capable of holding key/value pairs or typed records for each configuration concept.
- Reuses Audit Log (SF-24) for change records.

## Security and Isolation Requirements

- Only System Admin sessions may read or modify these pages.
- Health indicators must not leak sensitive details (e.g., raw connection strings).
- Configuration changes must take effect atomically and consistently across the platform.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
