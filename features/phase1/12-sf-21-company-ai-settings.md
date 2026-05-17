# SF-21 — Company AI Settings

## Purpose

Allow Company Admin users to configure and securely store AI provider settings at the company level.

## Summary

This sub-feature provides a company-level AI settings screen where the Company Admin can choose a provider, save an encrypted API key, choose allowed models, and set a monthly token limit and usage tracking preference. The API key is encrypted at rest and never returned in full to the frontend.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Provide a Company AI Settings screen accessible only to Company Admin.
- Capture and store: provider, encrypted_api_key, allowed_models, monthly_token_limit, usage_tracking_enabled.
- Encrypt the API key at rest using a server-side mechanism; never store plaintext.
- Display the API key as a masked value (e.g., sk-****abcd) when shown back to the user.
- Allow the Company Admin to update or remove the stored key.
- Apply the company's settings when the system performs AI operations on behalf of the company.

## Out of Scope

- Personal AI Settings (covered in SF-22).
- AI usage tracking and reporting (covered in SF-23).
- Switching between multiple keys per provider.
- AI provider selection at request time per user.

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation

## Independent Deployment Notes

Can be deployed independently. AI usage tracking integration is optional and can be wired in once SF-23 is available.

## User Stories

- As a Company Admin, I want to configure my company's AI provider and key so that AI features work for my team.
- As a security stakeholder, I want API keys to be encrypted and never shown in full so that they cannot be leaked.

## Acceptance Criteria

- A Company Admin can save AI settings with required fields validated.
- API keys are encrypted at rest; database inspection does not reveal the plaintext key.
- The frontend never receives the full API key after it has been saved.
- Non-admin users cannot view or change company AI settings.
- AI operations performed on behalf of the company use the configured settings.

## Data Requirements

- AI Settings: id, company_id, user_id (must be null for company settings), provider, encrypted_api_key, allowed_models, monthly_token_limit, usage_tracking_enabled, created_at, updated_at.
- Constraint: exactly one of company_id or user_id is set.

## Security and Isolation Requirements

- The encryption mechanism must use a key managed outside the database.
- API keys must never be written to logs.
- The settings record is scoped to the requesting user's company (per SF-14).
- Edit access is restricted to Company Admin (per SF-13).

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
