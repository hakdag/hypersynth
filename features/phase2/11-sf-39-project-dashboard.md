# SF-39 — Project Dashboard

## Purpose

Provide a per-project overview that summarizes task health, overdue work, and workload per user.

## Summary

This sub-feature adds a dashboard view on each project that aggregates tasks by status, surfaces overdue tasks, and computes workload per member. It is read-only and composes data from existing task and membership models; widgets gracefully omit themselves when their inputs are not yet available.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Provide a dashboard endpoint per project returning:
  - Counts of tasks grouped by status.
  - List or count of overdue tasks (when due dates exist).
  - Workload per user (open task count per member of the project).
- Limit dashboard access to project members and authorized roles (per SF-13 and SF-17).
- Ensure dashboard queries are efficient at scale.

## Out of Scope

- Customizable dashboards.
- Burndown or velocity charts.
- Cross-project dashboards (a company-wide dashboard is a future concern).
- Export to file.

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation
- SF-17 Project Membership

## Independent Deployment Notes

Ships independently. Widgets that rely on optional sub-features (overdue requires SF-31, status grouping reflects whatever status set is currently deployed) render based on available data.

## User Stories

- As a Project Manager, I want a quick view of the project’s health so that I can spot problems early.
- As a Contributor, I want to see workload per teammate so that I can balance work.

## Acceptance Criteria

- The dashboard returns the documented aggregations for the requested project.
- Users who are not authorized to view the project receive an authorization error.
- Aggregations match the underlying task data and are recomputed on each request (no stale snapshots).
- Empty projects render gracefully with zeroed metrics.

## Data Requirements

- No new persistent entities; relies on existing Task, Project_Membership, and (optionally) due date fields.
- Indexes from SF-38 are reused where applicable.

## Security and Isolation Requirements

- Dashboard data is scoped to the requested project and the requester’s company.
- Workload-per-user lists only include members of the project.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
