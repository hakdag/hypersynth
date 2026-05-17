# SF-14 — Company Data Isolation

## Purpose

Guarantee that company users can only access data that belongs to their own company, and that personal users cannot access company data.

## Summary

This sub-feature enforces tenant isolation at the data access layer. Every company-scoped entity must include a company_id, every personal-scoped entity must include an owner_user_id, and all backend queries must filter by the requesting identity. This is a cross-cutting capability that protects every existing and future company-scoped feature.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add company_id to all company-scoped entities: projects, features, tasks, documents, AI settings, AI usage, audit logs, invitations, project memberships.
- Add owner_user_id to personal-scoped entities for personal accounts.
- Enforce automatic filtering by company_id (or owner_user_id for personal accounts) on all read and write operations.
- Reject cross-tenant access attempts with a clear authorization error.
- Provide a single mechanism for resolving the active tenant from the user session.

## Out of Scope

- Sharing data between companies.
- Multi-company membership.
- System Admin cross-company access (handled in SF-18 and related admin features).

## Dependencies

- SF-11 Company Account Registration
- SF-13 User Roles and Authorization

## Independent Deployment Notes

Can be deployed alongside or immediately after company registration because every company-scoped entity must enforce isolation from the moment it exists. Existing single-tenant Phase 0 entities can be migrated to include the new fields with personal account semantics preserved.

## User Stories

- As a Company Admin, I want my company's data to remain private so that other companies cannot see or modify it.
- As a personal user, I do not want any company users to see my personal projects.

## Acceptance Criteria

- Every company-scoped entity has a non-null company_id.
- Every personal-scoped entity has a non-null owner_user_id.
- Read and write operations filter by the active tenant automatically.
- Attempts to access another tenant's records by id return an authorization error and do not leak data.
- A regression test or equivalent demonstrates that a user from Company A cannot access an entity from Company B.

## Data Requirements

- Add company_id column to all company-scoped entities.
- Add owner_user_id column to personal-scoped entities.
- For AI Settings: exactly one of company_id or user_id must be set.

## Security and Isolation Requirements

- Tenant filtering must be enforced on the backend, not on the frontend.
- Identifiers used in URLs must not be assumed to be authorized; the server must always re-check ownership.
- Bypassing isolation through bulk operations, search, or AI features must not be possible.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
