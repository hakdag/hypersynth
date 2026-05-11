# SF-20 — System Admin Dashboard: User Management

## Purpose

Allow the System Admin to view and manage user accounts across the platform.

## Summary

This sub-feature provides the user management area inside the System Admin dashboard. The admin can list and search users, view their account type, role, company affiliation, and status, and enable or disable users when necessary.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Provide a Users list page in the System Admin dashboard.
- Support search by email, name, username, and company.
- Display account type (Personal/Company), role for company users, company affiliation, and status.
- Allow the System Admin to enable or disable a user.
- Provide a clearly labeled action to reset user access (e.g., force re-authentication) without exposing the user's password.

## Out of Scope

- Editing user profile fields beyond status and access reset.
- Logging in as another user (admin impersonation is a future enhancement).
- Bulk user operations.

## Dependencies

- SF-18 System Admin Authentication
- SF-11 Company Account Registration
- SF-14 Company Data Isolation

## Independent Deployment Notes

Can be deployed independently from company management. Without SF-19, basic user listing and search still provide value.

## User Stories

- As a System Admin, I want to find any user quickly so that I can respond to support and security issues.
- As a System Admin, I want to disable a problematic user immediately so that they cannot continue to use the platform.

## Acceptance Criteria

- The System Admin can list and search users.
- The System Admin can view a user's account type, role, company affiliation, and status.
- The System Admin can change a user's status between Active and Disabled.
- A disabled user cannot log in or perform any authenticated action.
- The reset access action invalidates active sessions for the targeted user.

## Data Requirements

- Reuses the User and Company entities.

## Security and Isolation Requirements

- Only System Admin sessions may access user management screens.
- Password hashes must never be exposed through these screens or APIs.
- Disabling or resetting a user must take effect on the very next request from that user.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
