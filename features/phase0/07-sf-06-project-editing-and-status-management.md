# SF-06 — Project Editing and Status Management

## Purpose

Allow a user to update their project metadata, requirements, status, and AI API key.

## Summary

This sub-feature completes basic project maintenance. It should be deployable without feature, task, document, or AI execution modules.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Allow editing project name.
- Allow editing rich text project requirements.
- Allow updating project status among Pending, In Progress, and Done.
- Allow adding, replacing, or clearing the project AI API key.
- Validate required project name.
- Persist changes only for projects owned by the authenticated user.

## Out of Scope

- Requirement versioning.
- Approval workflow for AI-enhanced requirements.
- Project deletion.
- AI provider validation unless separately introduced.

## Dependencies

- SF-05 Project Listing and Detail View

## Independent Deployment Notes

Can be deployed as project maintenance. It does not require child object workflows to be available.

## User Stories

- As a user, I want to update project requirements so that the project description stays accurate.
- As a user, I want to change project status so that I can track high-level progress.

## Acceptance Criteria

- A user can update name, requirements, status, and AI API key for their own project.
- A project cannot be saved with an empty name.
- Status can only be one of the defined Phase 0 statuses.
- A user cannot edit a project owned by another user.
- AI API key can be omitted, replaced, or cleared.

## Data Requirements

- Updates Project: name, requirements, status, ai_api_key.
- Status values remain constrained to Pending, In Progress, Done.

## Security and Isolation Requirements

- Only the owner can update project fields.
- AI API key handling must avoid accidental display or logging.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.

