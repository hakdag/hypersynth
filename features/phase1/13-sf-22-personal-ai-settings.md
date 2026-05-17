# SF-22 — Personal Account AI Settings

## Purpose

Allow Personal Account users to configure and securely store AI provider settings at the user level.

## Summary

This sub-feature provides a personal AI settings screen where a Personal Account user can choose a provider, save an encrypted API key, choose allowed models, and set a monthly token limit and usage tracking preference. The behavior mirrors company AI settings but is scoped to a single user.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Provide a Personal AI Settings screen accessible only to the owning Personal Account user.
- Capture and store: provider, encrypted_api_key, allowed_models, monthly_token_limit, usage_tracking_enabled.
- Encrypt the API key at rest; never store plaintext.
- Display the API key as a masked value when shown back to the user.
- Allow the user to update or remove the stored key.
- Apply the personal settings when the system performs AI operations for that user.

## Out of Scope

- Company AI Settings (covered by SF-21).
- AI usage tracking and reporting (covered by SF-23).
- Sharing personal settings with other users.

## Dependencies

- SF-01 User Registration (Phase 0)
- SF-14 Company Data Isolation (for the personal-ownership filtering rules)

## Independent Deployment Notes

Can be deployed independently of SF-21. Personal users gain immediate value as soon as it ships.

## User Stories

- As a personal user, I want to configure my own AI provider and key so that AI features work for my projects.
- As a security stakeholder, I want personal API keys to be encrypted and never shown in full.

## Acceptance Criteria

- A Personal Account user can save AI settings with required fields validated.
- API keys are encrypted at rest; database inspection does not reveal the plaintext key.
- The frontend never receives the full API key after it has been saved.
- Other users cannot view or change another user's personal AI settings.
- AI operations performed for the user use the configured personal settings.

## Data Requirements

- AI Settings: id, company_id (must be null for personal settings), user_id, provider, encrypted_api_key, allowed_models, monthly_token_limit, usage_tracking_enabled, created_at, updated_at.
- Constraint: exactly one of company_id or user_id is set.

## Security and Isolation Requirements

- Encryption mechanism uses a key managed outside the database.
- API keys must never be written to logs.
- Access is strictly limited to the owning user.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
