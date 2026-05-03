# SF-12 — Task Editing and Status Management

## Purpose

Allow a user to update task details and task status.

## Summary

This sub-feature completes the basic manual task lifecycle for Phase 0.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Allow editing task title.
- Allow editing task description.
- Allow updating task status among Pending, In Progress, and Done.
- Validate required task fields according to Phase 0 rules.
- Preserve created_by origin when a task is edited.
- Persist changes only when the task belongs to the authenticated user through its parent hierarchy.

## Out of Scope

- Task deletion.
- Task assignment.
- Task comments.
- Task due dates.
- AI regeneration of existing tasks.

## Dependencies

- SF-11 Task Listing and Detail View

## Independent Deployment Notes

Can be deployed as task maintenance. It does not require document or AI features.

## User Stories

- As a user, I want to update task descriptions so that task details stay accurate.
- As a user, I want to update task status so that I can track progress.

## Acceptance Criteria

- A user can update title, description, and status for tasks in their own workspace.
- Status can only be Pending, In Progress, or Done.
- created_by is not changed by ordinary editing.
- A user cannot edit another user’s task.

## Data Requirements

- Updates Task: title, description, status.
- Preserves Task.created_by.

## Security and Isolation Requirements

- Parent ownership must be validated before task update.
- Unauthorized update attempts must be blocked.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.

