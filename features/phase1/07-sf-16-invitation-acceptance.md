# SF-16 — Invitation Acceptance and Onboarding

## Purpose

Allow an invited user to accept an invitation, create or link an account, and join the inviting company with the assigned role.

## Summary

This sub-feature handles the user-facing side of invitations. When the invited person opens the invitation link, the system validates the token, then either presents a registration page (new user) or a login + confirm page (existing user). On success, the user is linked to the company with the invited role, optionally bound to a project, and the invitation is marked Accepted.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Implement a public route that accepts an invitation token.
- Validate token validity, expiration, and status (only Pending is acceptable).
- For a new user: display a registration form pre-filled with the invited email; create the user with account_type Company, role from the invitation, status Active, and link to the company.
- For an existing user with the invited email: require login and explicit acceptance, then link the user to the company and assign the invited role (subject to the Phase 1 single-company rule).
- If the invitation includes a project_id, bind the user to that project.
- Mark the invitation as Accepted with accepted_at set on success.
- Provide clear error states for invalid, expired, cancelled, or already-accepted invitations.

## Out of Scope

- Multi-company membership beyond Phase 1's one-company rule.
- Project-level access control beyond simple binding (handled in SF-17).
- Re-invitation flows.

## Dependencies

- SF-15 Team Member Invitations
- SF-01 User Registration (Phase 0) for the registration form pattern

## Independent Deployment Notes

Can be deployed once SF-15 stores invitations. Project binding can be best-effort: if SF-17 Project Membership is not yet deployed, the project_id can simply be stored on the user-project link and become effective when project memberships are introduced.

## User Stories

- As an invited user, I want to open the invitation link, register, and immediately be part of the company so that I can start working.
- As an existing user, I want to accept an invitation by logging in so that I am added to the inviting company without creating a duplicate account.

## Acceptance Criteria

- A valid invitation link leads to a working registration or accept-and-link page.
- An expired, cancelled, or already-accepted invitation shows a clear error and cannot be reused.
- A new user created through invitation has account_type Company, the correct role, and is linked to the inviting company.
- An existing user, after accepting, is linked to the inviting company with the correct role.
- If the invitation included a project, the accepting user is bound to that project.
- The invitation status becomes Accepted and accepted_at is set.

## Data Requirements

- Reuses Invitation, User, and (optionally) Project Membership entities.
- No duplicate User records are created for an existing email.

## Security and Isolation Requirements

- Tokens must be validated server-side; the URL alone must not grant access without server validation.
- Acceptance must be a one-time operation; replays must fail.
- Sessions created via invitation acceptance must follow the same security rules as normal logins.
- An invitation must not allow privilege escalation beyond the role specified at invitation time.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
