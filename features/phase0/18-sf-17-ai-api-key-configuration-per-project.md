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
- The full saved key is not displayed in ordinary views.
- A user cannot manage a key for another user’s project.

## Data Requirements

- Uses Project.ai_api_key or equivalent secure storage field.
- Project-level key is optional.

## Security and Isolation Requirements

- AI API key must be protected as sensitive data.
- The key must not appear in logs, errors, or ordinary read-only screens.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.

