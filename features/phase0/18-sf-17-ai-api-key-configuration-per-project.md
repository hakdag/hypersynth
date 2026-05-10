# SF-17 — AI API Key Configuration Per Project

## Purpose

Allow each project to hold optional AI provider credentials for AI-assisted workflows.

## Summary

This sub-feature formalizes AI API key management as a deployable capability independent from actual AI execution.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Allow adding an AI API key to a project.
- Allow replacing an existing AI API key.
- Allow clearing an AI API key.
- Show whether a project has an AI API key configured.
- Avoid displaying the full key after it has been saved.
- Store API keys encrypted at rest, never as plaintext.
- Ensure only the project owner can manage the key.

## Out of Scope

- Calling an AI provider.
- Testing key validity unless separately introduced.
- Multiple AI providers or multiple keys.
- Organization-level key management.

## Dependencies

- SF-06 Project Editing and Status Management

## Independent Deployment Notes

Can be deployed before AI features. It prepares projects for later AI-assisted actions.

## User Stories

- As a user, I want to configure an AI API key per project so that AI features can use project-specific credentials.
- As a user, I want the saved key hidden so that it is not accidentally exposed.

## Acceptance Criteria

- A user can add, replace, and clear the key for their own project.
- A project can exist without an AI API key.
- The project detail view indicates whether a key exists.
- The full saved key is never returned after save.
- Read responses return only key presence, such as `hasAiApiKey`, and may include a masked suffix such as `sk-...1234`.
- A user cannot manage a key for another user’s project.
- API key create, replace, clear, and runtime use events are audited without recording the key value.

## Data Requirements

- Store only `Project.encrypted_api_key` or an equivalent encrypted storage field.
- Do not store a plaintext `ai_api_key` value.
- Project-level key is optional.
- Store non-sensitive display metadata only when needed, such as key presence and masked suffix.
- Add an application secret key to the `.env` file and use it to encrypt and decrypt project API keys.

## Security and Isolation Requirements

- AI API keys must be protected as sensitive data and encrypted at rest.
- API keys must never be stored as plaintext.
- API keys must never appear in logs, request-body logs, errors, provider error messages, ordinary read-only screens, or returned API responses.
- Decrypted API keys may exist only inside the server-side AI execution path and only for the owning project/user.
- Provider errors must be sanitized before logging or returning them, because upstream responses may include request details.
- Create, replace, clear, and runtime use events must be audited without recording plaintext, ciphertext, or decrypted values.

## Configuration Requirements

- `.env` must provide a secret encryption key for API key encryption.
- The encryption key must not be committed to source control.
- Missing or invalid encryption configuration must prevent API key storage and runtime AI execution.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- Stored API keys are encrypted at rest using the configured `.env` secret key.
- Plaintext API keys are not returned, logged, audited, or persisted.
- Audit records prove key lifecycle and runtime use events without exposing the key.
- The feature can be tested with clear pass/fail acceptance criteria.

