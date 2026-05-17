# SF-18 — System Admin Authentication

## Purpose

Provide a secure way for a single platform-wide System Admin to authenticate using credentials configured in the environment.

## Summary

This sub-feature introduces the System Admin identity and login. The Phase 1 approach reuses the regular login screen but detects the System Admin email and validates against a hashed password stored in environment variables. Successful login creates a System Admin session distinct from regular user sessions and routes the admin to the System Admin dashboard.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Read System Admin configuration from environment variables: SYSTEM_ADMIN_EMAIL, SYSTEM_ADMIN_PASSWORD_HASH, SYSTEM_ADMIN_ENABLED.
- Detect System Admin login from the regular login screen by matching the submitted email.
- Verify the submitted password against the configured password hash.
- On success, create a System Admin session distinguishable from regular user sessions.
- Redirect successful System Admin logins to the System Admin dashboard.
- Log every System Admin login attempt (success and failure) with timestamp, IP, and user agent.
- Treat the System Admin as separate from any company; never expose company workspaces through this session.

## Out of Scope

- Dedicated /admin/login route (future enhancement).
- Multi-factor authentication for System Admin (future enhancement).
- Multiple System Admin accounts.
- Storing System Admin in the database.

## Dependencies

- SF-00 Project Initialization and Application Shell (Phase 0) for the login screen pattern

## Independent Deployment Notes

Can be deployed even before any System Admin dashboard pages exist. In that case, a successful System Admin login lands on a placeholder dashboard. The authentication mechanism still adds value by establishing the secure entry point for later admin features.

## User Stories

- As a platform operator, I want to log in as System Admin using configured credentials so that I can access platform-wide tools.
- As a security stakeholder, I want all System Admin login attempts logged so that suspicious activity is detectable.

## Acceptance Criteria

- A user submitting the configured System Admin email and the correct password is authenticated as System Admin.
- A System Admin session is distinguishable from any regular user session.
- A successful System Admin login redirects to the System Admin dashboard (or placeholder).
- Incorrect System Admin password fails with the same generic error as a normal failed login.
- Every System Admin login attempt is logged with timestamp, IP, and user agent.
- If SYSTEM_ADMIN_ENABLED is false or values are missing, System Admin login is unavailable and clearly disabled.

## Data Requirements

- No new database entity is required for the System Admin identity in Phase 1.
- Login attempt logs may reuse the audit log entity (see SF-24) where audit logging is available; otherwise a minimal local log is acceptable.

## Security and Isolation Requirements

- The System Admin password must be stored only as a hash; plaintext is forbidden.
- The .env file must never be committed to source control.
- System Admin sessions must never grant access to any company-scoped data path that bypasses isolation rules.
- Login responses must not reveal whether an email is the System Admin email.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
