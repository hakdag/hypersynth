# SF-19 — System Admin Dashboard: Company Management

## Purpose

Allow the System Admin to view, search, and manage companies registered on the platform.

## Summary

This sub-feature provides the company management area inside the System Admin dashboard. The admin can list and search companies, view details, see basic counts (users, projects, documents), and enable or disable a company.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Provide a Companies list page in the System Admin dashboard.
- Support search by company name and email.
- Provide a company detail page showing all stored fields, current status, and aggregate counts: total users, total projects, total documents.
- Show a high-level AI usage summary if AI usage tracking is available; otherwise show a clearly empty state.
- Allow the System Admin to set company status to Active or Disabled.
- Enforce that disabled companies cannot be used by their users until re-enabled (login or session checks reject access).

## Out of Scope

- Editing the company profile fields on behalf of the company (the Company Admin owns that flow).
- Deleting companies.
- Cross-company data merge or transfer.

## Dependencies

- SF-18 System Admin Authentication
- SF-11 Company Account Registration
- SF-14 Company Data Isolation

## Independent Deployment Notes

Can be deployed independently from other admin pages. The AI usage summary can degrade gracefully to a placeholder when SF-23 is not yet deployed.

## User Stories

- As a System Admin, I want to find any company quickly so that I can investigate issues.
- As a System Admin, I want to disable a company that violates terms so that its users lose access immediately.

## Acceptance Criteria

- The System Admin can list and search companies.
- The System Admin can open a company detail page and see all fields, status, and aggregate counts.
- The System Admin can change company status between Active and Disabled.
- A user from a disabled company cannot access company workspaces or perform company-scoped actions.
- All company management actions are recorded in audit logs when audit logging is available.

## Data Requirements

- Reuses Company, User, Project, Document entities.
- No new entity introduced.

## Security and Isolation Requirements

- Only System Admin sessions may access company management screens.
- Status changes must be applied atomically and reflected in subsequent authorization checks.
- The system must not expose company-scoped business data (e.g., project content) through this screen beyond the documented summary fields.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
