# SF-13 — User Roles and Authorization

## Purpose

Introduce role-based access control for company users and enforce permissions consistently across all protected actions.

## Summary

This sub-feature defines the role model for Phase 1 (Company Admin, Project Manager, Contributor, Viewer) and provides a centralized authorization mechanism that protected backend actions can rely on. It does not, by itself, add new screens; it underpins existing and future screens by enforcing what each role may do.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Define the four company roles: Company Admin, Project Manager, Contributor, Viewer.
- Define the action-to-role permission matrix as specified in the Phase 1 FRD section 5.3.
- Provide a centralized authorization check used by all protected backend actions.
- Apply role-based gating to existing company-scoped actions (project, feature, task, document operations) where relevant.
- Surface role information on the user session so the frontend can hide actions a user cannot perform (in addition to backend enforcement).

## Out of Scope

- Custom roles defined per company.
- Project-level role overrides beyond the project membership concept (handled in SF-17).
- System Admin role (handled in SF-18).
- Multi-company role membership.

## Dependencies

- SF-11 Company Account Registration

## Independent Deployment Notes

Can be deployed even before invitations exist, because the Company Admin created at registration is the only company user. The authorization layer is exercised against that single user and can be extended seamlessly when more roles appear.

## User Stories

- As a Company Admin, I want roles to be enforced so that team members only do what their role allows.
- As a developer, I want one authorization mechanism so that protected actions are not accidentally left unguarded.

## Acceptance Criteria

- Each company user record carries a role from the defined set.
- Backend actions consistently check the role of the requesting user before execution.
- Actions denied by role return a clear authorization error and do not modify data.
- The permission matrix from the FRD is implemented faithfully.
- Frontend may hide controls based on role, but the backend remains the source of truth.

## Data Requirements

- User.role field uses one of: Company Admin, Project Manager, Contributor, Viewer.
- Role is required for users with account_type Company.

## Security and Isolation Requirements

- Authorization checks must run on the backend on every protected request.
- A user must not be able to perform actions outside the permissions of their role by manipulating client-side calls.
- Role values must come from a controlled set; arbitrary role strings must be rejected.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
