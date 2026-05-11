# SF-27 — System Admin Dashboard: Invitation Monitoring

## Purpose

Allow the System Admin to monitor invitations across all companies and intervene on suspicious ones.

## Summary

This sub-feature provides an invitations view inside the System Admin dashboard. The admin can browse pending and expired invitations, filter by company, and cancel invitations that look suspicious.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Provide an invitations page in the System Admin dashboard.
- List pending and expired invitations with company, inviter, invited email, role, status, expiration, and creation timestamp.
- Filter by company, status, and date range.
- Allow the System Admin to cancel a Pending invitation, setting status to Cancelled.
- Record cancellations in the audit log when audit logging is available.

## Out of Scope

- Creating invitations on behalf of companies.
- Resending invitations.
- Changing the role or project on an invitation.

## Dependencies

- SF-18 System Admin Authentication
- SF-15 Team Member Invitations

## Independent Deployment Notes

Can be deployed any time after invitations exist. Without SF-15, this sub-feature has no data to show and should not be enabled.

## User Stories

- As a System Admin, I want to see pending invitations across the platform so that I can detect abuse.
- As a System Admin, I want to cancel a suspicious invitation so that it cannot be accepted.

## Acceptance Criteria

- The System Admin can list pending and expired invitations with the documented fields.
- Filters work for company, status, and date range.
- Cancelling a Pending invitation sets its status to Cancelled and prevents acceptance.
- Already-accepted or expired invitations cannot be cancelled.
- Only System Admin sessions can access this view.

## Data Requirements

- Reuses the Invitation entity from SF-15.

## Security and Isolation Requirements

- Only System Admin sessions can access this view.
- Invitation tokens are not displayed in the UI; only the metadata required for triage is shown.
- Cancellation must take effect immediately for any subsequent acceptance attempt.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
