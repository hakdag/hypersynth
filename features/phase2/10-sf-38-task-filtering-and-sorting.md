# SF-38 — Task Filtering and Sorting

## Purpose

Provide consistent, indexed filtering and sorting of task lists so that users can find and order work efficiently.

## Summary

This sub-feature delivers backend support for filtering tasks by assignee, status, priority, due date, and labels, and sorting by due date, priority, created_at, and updated_at. Each filter and sort dimension is optional; the feature degrades gracefully when an underlying field-producing sub-feature is not yet deployed.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Support filters: assignee, status, priority, due_date (range and overdue), labels.
- Support sorts: due_date, priority, created_at, updated_at.
- Allow combining multiple filters in a single request.
- Apply pagination consistently.
- Add database indexes for the supported filters and sorts.

## Out of Scope

- Full-text search.
- Saved views.
- Frontend filter UI design (covered separately).
- Cross-project aggregations beyond simple filtering (the dashboard is SF-39).

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation

## Independent Deployment Notes

Filters degrade based on which fields exist. For example, the priority filter is ignored if SF-30 is not yet deployed, and label filters apply only when SF-33 is present. The feature still ships with whatever subset of fields exists at deploy time.

## User Stories

- As a Contributor, I want to filter my task list by status and priority so that I can focus on what matters.
- As a Project Manager, I want to view all overdue tasks so that I can address slippage.

## Acceptance Criteria

- All listed filter and sort parameters are accepted and validated.
- Combined filters compose using logical AND.
- Sorting is deterministic; ties are broken by id.
- Pagination produces stable results for the same filter and sort.
- Queries use indexes; documented filters scale on large task counts.

## Data Requirements

- Indexes for assignee_user_id, status, priority, due_date, created_at, updated_at, and the Task_Label join.

## Security and Isolation Requirements

- Filtering must never return tasks outside the requester’s authorized scope.
- Filter parameters cannot be used to enumerate inaccessible data.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
