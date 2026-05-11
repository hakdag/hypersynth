# SF-15 — Team Member Invitations

## Purpose

Allow Company Admin users (and Project Managers when permitted) to invite team members to join their company by email.

## Summary

This sub-feature provides the invitation creation flow. The inviter selects the email, role, and optional project assignment for the new team member. The system stores an invitation record with a secure token and sends an invitation email with a unique link. Acceptance is handled in SF-16.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Provide an "Invite team member" screen accessible to Company Admin (and Project Manager per the permission matrix).
- Capture invitation fields: email, role, optional project, optional message.
- Generate a cryptographically secure invitation token.
- Persist an Invitation record with status Pending and an expiration timestamp.
- Send an invitation email containing the company name, inviter name, invited role, optional project name, invitation link, and expiration information.
- Allow the inviter to view a list of invitations they created with their statuses.
- Allow the inviter to cancel a pending invitation, setting its status to Cancelled.
- Validate that the email format is valid and that there is no other Pending invitation for the same email in the same company.

## Out of Scope

- Invitation acceptance and registration completion (covered by SF-16).
- Bulk CSV invitations.
- Reminders for unaccepted invitations.
- Sending re-invitations after acceptance.

## Dependencies

- SF-11 Company Account Registration
- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation

## Independent Deployment Notes

Can be deployed before SF-16 is complete only if pending invitations are visible internally for testing; for end-user value, deploying together with SF-16 is recommended. Even without SF-17 Project Membership, project assignment can be captured on the invitation and applied later.

## User Stories

- As a Company Admin, I want to invite a teammate by email so that they can join my company workspace.
- As a Project Manager, I want to invite a contributor to a specific project so that they can start contributing immediately.
- As an inviter, I want to see and cancel pending invitations so that I keep the team list clean.

## Acceptance Criteria

- An authorized user can submit an invitation with valid fields.
- An Invitation record is created with status Pending, a secure token, and a future expiration.
- An invitation email is sent containing the required information and a unique link.
- The inviter sees their pending invitations and can cancel any of them.
- A Pending invitation cannot be created for an email that already has a Pending invitation in the same company.
- Unauthorized users cannot create or cancel invitations.

## Data Requirements

- Invitation: id, invitation_token, company_id, project_id nullable, invited_email, invited_role, invited_by_user_id, status (Pending|Accepted|Expired|Cancelled), expires_at, accepted_at nullable, created_at.
- invitation_token must be a cryptographically secure UUID/token.

## Security and Isolation Requirements

- Invitation tokens must be hard to guess and must never be exposed in logs.
- Invitation operations are scoped to the inviter's company (per SF-14).
- Cancelled or expired invitations must not be reusable.
- Invitations must not reveal whether the email is already a user of another company.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
