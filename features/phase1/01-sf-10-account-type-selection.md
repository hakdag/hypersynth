# SF-10 — Account Type Selection at Registration

## Purpose

Allow new users to choose between a Personal Account and a Company Account at the start of registration so the platform can support both individual usage and multi-user company workspaces.

## Summary

This sub-feature introduces a single decision point in the registration flow: a screen that asks the user to pick an account type. Selecting Personal Account routes the user into the Phase 0 personal registration flow. Selecting Company Account routes the user into the company registration flow. This sub-feature does not implement company registration itself; it only adds the account-type fork.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add an account type selection screen at the beginning of registration.
- Provide two clear options: Personal Account and Company Account.
- Route Personal Account selection to the existing personal registration flow from Phase 0.
- Route Company Account selection to the company registration flow when it is available, or to a clearly labeled placeholder/unavailable state when not.
- Persist the selected account type as part of the resulting user record.

## Out of Scope

- Implementation of company registration fields and company creation.
- Conversion between Personal Account and Company Account after registration.
- OAuth/social login.
- Email verification.

## Dependencies

- SF-01 User Registration (Phase 0)

## Independent Deployment Notes

Can be deployed before company registration is implemented by routing the Company Account choice to a clearly labeled "coming soon" or disabled state. This still provides value because the platform now expresses the dual-account-type concept and continues to deliver the Phase 0 personal registration path.

## User Stories

- As a new user, I want to choose between a Personal Account and a Company Account so that I can register the way that fits my situation.
- As a product owner, I want a clear account type fork so that future company features can attach to a stable starting point.

## Acceptance Criteria

- The registration entry point presents both options: Personal Account and Company Account.
- Selecting Personal Account leads to the Phase 0 personal registration flow.
- Selecting Company Account leads to the company registration flow (or to a clearly labeled unavailable state if it is not yet enabled).
- The created user record reflects the chosen account type.
- The user cannot proceed to registration fields until an account type is selected.

## Data Requirements

- User: account_type field with values Personal or Company.
- Default value is not assumed; the user must make an explicit choice.

## Security and Isolation Requirements

- Account type selection alone must not grant any access; downstream registration steps remain responsible for identity validation.
- The selected account type must be set on the server side based on the chosen flow, not blindly trusted from the client form.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
