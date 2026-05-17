# SF-30 — Task Priority

## Purpose

Introduce a controlled priority level on every task so that work can be ranked and surfaced consistently.

## Summary

This sub-feature defines a four-value priority enum (Low, Medium, High, Critical) with Medium as the default, exposes it on task create and edit, and emits a change event for activity logging. Priority becomes the foundation for later filtering, sorting, and dashboard signals but does not require those features to deliver value on its own.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add a required `priority` field to the task entity with values Low, Medium, High, Critical.
- Default newly created tasks to Medium.
- Allow priority to be set on create and changed on edit by users with edit permission.
- Display priority on task views and lists.
- Emit a priority change event for activity logging.

## Out of Scope

- Custom or company-defined priority levels.
- Numeric priority scores.
- Automatic priority escalation.
- Filtering/sorting by priority (covered by SF-38).

## Dependencies

- SF-13 User Roles and Authorization

## Independent Deployment Notes

Can ship independently of assignment, dependencies, labels, or dashboards. Existing tasks created before deployment must be backfilled to Medium.

## User Stories

- As a Project Manager, I want to mark tasks by priority so that the team can focus on what matters most.
- As a Contributor, I want to see task priority at a glance so that I can sequence my work.

## Acceptance Criteria

- Every task has a priority value drawn from the controlled set.
- Newly created tasks default to Medium.
- Priority can be changed through the edit flow.
- Invalid priority values are rejected.
- Priority changes are captured as activity-log-ready events.

## Data Requirements

- Task.priority is a non-null enum of {Low, Medium, High, Critical}.
- Existing rows must be migrated to Medium at deployment.

## Security and Isolation Requirements

- Only users with edit rights on the task may change priority (per SF-13 matrix).
- Priority values are validated against the controlled enum on every write.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
