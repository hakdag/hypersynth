# SF-32 — Extended Task Status Model

## Purpose

Expand the task status model to reflect real execution stages and enforce sensible transitions.

## Summary

This sub-feature replaces the minimal Phase 1 status set with the full set: Pending, In Progress, Blocked, In Review, Done, Cancelled. Pending is the default. Transitions are recorded as status change events. The interaction with task dependencies (preventing Done while blocked) is delivered as a soft guard here and becomes hard-enforced once SF-37 is deployed.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Define task statuses: Pending, In Progress, Blocked, In Review, Done, Cancelled.
- Default new tasks to Pending.
- Allow status changes through the edit flow.
- Mark Done and Cancelled as terminal statuses (used by SF-31 overdue computation).
- Emit status change events for activity logging.
- Provide a hook point for SF-37 to reject Done transitions when dependencies are unresolved.

## Out of Scope

- Workflow rules per project or per company.
- Custom statuses.
- Automatic status transitions based on assignment or comments.

## Dependencies

- SF-13 User Roles and Authorization

## Independent Deployment Notes

Ships independently. The dependency-aware Done guard is only active once SF-37 is deployed. Until then, Done is permitted regardless of dependencies and the system remains internally consistent.

## User Stories

- As a Contributor, I want to mark a task In Progress, Blocked, or In Review so that the team has accurate visibility.
- As a Project Manager, I want a Cancelled status so that abandoned work is removed from active views without being deleted.

## Acceptance Criteria

- Every task has a status from the controlled set.
- New tasks default to Pending.
- Invalid status values are rejected.
- Terminal statuses are recognized consistently across the system.
- Status changes produce activity-log-ready events.

## Data Requirements

- Task.status is a non-null enum of {Pending, In Progress, Blocked, In Review, Done, Cancelled}.
- Existing rows must be migrated to a sensible default (e.g., Pending) at deployment.

## Security and Isolation Requirements

- Only users with edit rights on the task may change status (per SF-13).
- Status values are validated against the controlled enum on every write.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
