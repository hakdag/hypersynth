# SF-40 — Feature Progress and Grouping

## Purpose

Surface a feature’s child tasks in a grouped view and report completion progress so that progress is visible at the feature level.

## Summary

This sub-feature enhances the feature detail view to list its tasks grouped by status and to display a progress percentage computed as done tasks divided by total tasks. It introduces no new persistent entities and consumes data from existing task models.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Group a feature’s tasks by status on the feature detail view.
- Compute and display progress as `done_tasks / total_tasks`, expressed as a percentage with no division-by-zero error when total is zero.
- Use the deployed status set to determine what counts as done.
- Respect existing visibility rules; only tasks the requester can see are counted.

## Out of Scope

- Weighted progress (e.g., by estimate or priority).
- Multi-feature aggregate progress.
- Burndown over time.
- Editing tasks from within the grouped view (the existing task edit flows remain authoritative).

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation

## Independent Deployment Notes

Ships independently. If SF-32 is deployed, the grouping uses the full status set; otherwise it uses whatever status values exist. Progress remains well-defined as long as a Done status exists.

## User Stories

- As a Project Manager, I want to see at a glance how much of a feature is complete so that I can report status.
- As a Contributor, I want my tasks for a feature grouped by status so that I can quickly find what to do next.

## Acceptance Criteria

- The feature detail view shows tasks grouped by their status.
- Progress percentage is correctly computed and rounded for display.
- An empty feature shows zero tasks and a 0% progress (or an explicit empty state) without errors.
- Users see only the tasks they are authorized to view; progress is computed against the authorized set.

## Data Requirements

- No new entities. Reads from the existing Task entity by feature_id.
- Optional index on (feature_id, status) to keep grouping efficient at scale.

## Security and Isolation Requirements

- Grouped views and progress numbers respect company and project membership rules.
- Progress must not leak the existence of unauthorized tasks; the denominator only includes authorized tasks.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
