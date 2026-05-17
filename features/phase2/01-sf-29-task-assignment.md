# SF-29 — Task Assignment

## Purpose

Allow tasks to be assigned to a specific user so that ownership is explicit and accountable.

## Summary

This sub-feature adds an optional assignee on each task and the rules and behaviors that govern who can be assigned. Assignment can be set on create or edit, may be cleared, and every change is captured as an assignment event that downstream activity logging can consume.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add an optional `assignee_user_id` to the task entity.
- Allow setting and clearing the assignee on task create and task edit.
- Validate that the assignee shares the same company (or personal account) as the task owner.
- Validate that the assignee is a project member when the task belongs to a project (per SF-17).
- Surface the assignee on task views and lists.
- Emit a structured assignment change event so SF-36 Activity Logs (when present) can record it; if activity logs are not yet deployed, the event is simply unused.

## Out of Scope

- Multiple assignees per task.
- Auto-assignment rules.
- Notification of the assignee (deferred to a future notifications feature).
- Filtering/sorting by assignee (covered by SF-38).

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation
- SF-17 Project Membership

## Independent Deployment Notes

Can ship independently of other Phase 2 sub-features. Tasks without an assignee remain valid. The assignment change event is fire-and-forget so activity logging is not a hard prerequisite.

## User Stories

- As a Project Manager, I want to assign a task to a teammate so that ownership is clear.
- As a Contributor, I want to see which tasks are assigned to me so that I know what to work on.
- As a Company Admin, I want assignment limited to people who belong to the same workspace and project so that data isolation is preserved.

## Acceptance Criteria

- A task can be created with or without an assignee.
- A task’s assignee can be changed or cleared by a user with permission to edit the task.
- Assigning a user from a different company is rejected.
- For project tasks, assigning a non-member of that project is rejected.
- Every successful assignment change produces an event suitable for activity logging.

## Data Requirements

- Task.assignee_user_id is nullable and references a user in the same company or the same personal account.

## Security and Isolation Requirements

- Cross-company assignment is rejected at the backend.
- Authorization for changing assignee follows the task’s edit permission rules.
- Assignment changes cannot be made by clients to circumvent role-based controls (per SF-13).

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
