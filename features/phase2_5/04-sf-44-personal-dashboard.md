# SF-44 — Personal Dashboard

## Purpose

Give every user a personalized view of their own work: what they have open, what is overdue, what is blocking them, their workload relative to teammates, and their recent throughput.

## Summary

This sub-feature introduces a per-user dashboard accessible to every authenticated user. It groups the user's open tasks by status and priority, lists overdue tasks, surfaces blockers (in both directions through dependencies), shows workload index versus team average, displays a 4-week throughput trend, and lists recent mentions and unread comment activity.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add a Personal Dashboard accessible to every authenticated user.
- Sections:
  - My open tasks grouped by status and priority
  - My overdue tasks
  - Tasks blocking me and tasks I am blocking
  - My workload index vs team average
  - My throughput trend (last 4 weeks)
  - Mentions and unread comment activity
- Data is restricted to tasks the user is assigned to or otherwise authorized to view.

## Out of Scope

- Project- and company-level dashboards (SF-42, SF-43).
- Notifications / email digests.
- Personal goal setting or custom KPIs.

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation
- SF-29 Task Assignment
- SF-37 Task Dependencies (blockers section; gracefully omitted if not yet deployed)
- SF-41 Metrics Snapshot Pipeline (throughput trend; gracefully omitted if not yet deployed)

## Independent Deployment Notes

Ships independently. Each section degrades gracefully when its required input sub-feature is not yet deployed.

## User Stories

- As a Contributor, I want to see all my open and overdue work in one place.
- As a Contributor, I want to know which tasks are blocked or blocking others so I can resolve dependencies.
- As any user, I want to see how my workload compares to teammates.

## Acceptance Criteria

- Every authenticated user can open the Personal Dashboard.
- All listed sections render with data scoped to the requester.
- Tasks belonging to other users or other companies never appear.
- Sections without required data render an empty state rather than failing.

## Data Requirements

- No new persistent entities.
- Reads from existing Task, Dependency, Comment, Mention, and `metric_snapshot` (SF-41).

## Security and Isolation Requirements

- The dashboard returns only data the requester is authorized to read.
- Backend authorization is enforced on every read.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
