# SF-31 — Task Due Dates and Overdue Tracking

## Purpose

Allow tasks to carry an optional deadline and surface overdue status so that work can be planned against time.

## Summary

This sub-feature adds optional due date and due time fields to a task, computes overdue dynamically against the current time, and exposes overdue as a derived view-time property. It does not modify task lifecycle on its own; later sub-features may consume overdue for filtering and dashboards.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add nullable `due_date` and nullable `due_time` to the task entity.
- Allow setting and clearing both fields on create and edit.
- Compute `is_overdue` dynamically (true when the deadline is in the past and the task is not in a terminal status).
- Expose `is_overdue` on task views.
- Emit due-date change events for activity logging.

## Out of Scope

- Recurring due dates.
- Time zones beyond a single configured workspace zone (kept simple here).
- Reminder notifications.
- Filtering by overdue (covered by SF-38).

## Dependencies

- SF-13 User Roles and Authorization

## Independent Deployment Notes

Can ship without status-model changes; the overdue flag simply checks the current status against the existing terminal status set. If SF-32 is later deployed, the terminal status set used here automatically expands.

## User Stories

- As a Project Manager, I want to set due dates so that the team works against a schedule.
- As a Contributor, I want overdue tasks to be visibly flagged so I can prioritize accordingly.

## Acceptance Criteria

- A task can be created or edited with no due date, a date only, or a date and time.
- Clearing the due date is supported.
- Overdue is computed at read time and does not require a background job.
- A task in a terminal status is never reported as overdue.
- Due date changes produce activity-log-ready events.

## Data Requirements

- Task.due_date is a nullable date.
- Task.due_time is a nullable time.
- Both must be writable independently, subject to the rule that due_time without due_date is rejected.

## Security and Isolation Requirements

- Only users with edit rights on the task may change due fields.
- Overdue computation must respect the task’s visibility rules and never leak existence to unauthorized users.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
