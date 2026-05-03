# SF-04 — Project Creation

## Purpose

Allow an authenticated user to create a project in their private workspace.

## Summary

This sub-feature introduces the root business object: Project. It should be deliverable without feature, task, document, or AI workflows.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Provide a create project screen or action.
- Capture project name as required.
- Capture rich text project requirements as optional content.
- Set project status to Pending by default.
- Allow optional AI API key to be stored per project.
- Associate the project with the authenticated user.
- Show validation feedback for missing or invalid required fields.

## Out of Scope

- Project editing after creation.
- Project deletion.
- Feature and task management.
- AI calls using the API key.
- Document upload.

## Dependencies

- SF-03 Personal Workspace and Data Isolation

## Independent Deployment Notes

Can be deployed once authentication and isolation exist. Users can create project records even if project editing and child objects are delivered later.

## User Stories

- As an authenticated user, I want to create a project so that I can start organizing work.
- As a user, I want to optionally enter project requirements so that I can document the project context from the beginning.

## Acceptance Criteria

- A project cannot be created without a name.
- A project can be created without requirements.
- A project can be created without an AI API key.
- New project status defaults to Pending.
- Created project belongs only to the authenticated user.
- The user can see confirmation after successful creation.

## Data Requirements

- Project: id, user_id, name, requirements, status, ai_api_key.
- Status default: Pending.

## Security and Isolation Requirements

- Project ownership is assigned from the authenticated session, not from user-submitted ownership fields.
- AI API key must not be exposed to other users.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.

