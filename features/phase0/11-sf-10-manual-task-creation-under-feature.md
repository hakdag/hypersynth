# SF-10 — Manual Task Creation Under Feature

## Purpose

Allow a user to manually create tasks within a feature.

## Summary

This sub-feature introduces the third level of the work breakdown: Project → Feature → Task. It is independent from AI-generated task creation.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Provide a create task action from a feature detail view.
- Capture task title.
- Capture task description.
- Set task status to Pending by default.
- Set created_by to User or equivalent manual-origin marker.
- Associate the task with the selected feature.
- Ensure parent feature belongs to the authenticated user through its project.

## Out of Scope

- AI-generated task creation.
- Task editing after creation.
- Task deletion.
- Advanced task assignment or scheduling.

## Dependencies

- SF-08 Feature Listing and Detail View

## Independent Deployment Notes

Can be deployed as manual task creation. It does not require AI integration.

## User Stories

- As a user, I want to create tasks manually so that I can break a feature into actionable work.
- As a user, I want manually created tasks marked as user-created so that I can distinguish them from AI-generated tasks later.

## Acceptance Criteria

- A task can be created under a feature owned by the authenticated user.
- New task status defaults to Pending.
- Manual task origin is stored as User or equivalent.
- Task is linked to the selected feature.
- A user cannot create a task under another user’s feature.

## Data Requirements

- Task: id, feature_id, title, description, status, created_by.
- created_by default for this feature: User.
- Status default: Pending.

## Security and Isolation Requirements

- Parent feature and project ownership must be validated.
- Task ownership is derived from the parent feature and project.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.

