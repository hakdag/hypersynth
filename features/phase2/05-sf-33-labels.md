# SF-33 — Labels

## Purpose

Provide reusable, color-coded tags that can be applied to tasks to support categorization.

## Summary

This sub-feature introduces a Label entity scoped to a company or personal account, a management screen for creating and editing labels, and a many-to-many relationship to tasks. Labels become a building block for filtering (SF-38) but deliver value standalone through visual categorization.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add a Label entity with id, name, color (hex), and scope (company_id OR user_id).
- Provide CRUD for labels within the user’s scope, gated by SF-13 roles.
- Enforce unique label name per scope.
- Provide a many-to-many Task_Label association.
- Allow applying and removing labels on task create and edit.
- Display labels on task views.

## Out of Scope

- Cross-scope labels.
- Hierarchical labels or label groups.
- Filtering by label (covered by SF-38).
- Bulk relabeling tools.

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation

## Independent Deployment Notes

Labels and Task_Label can be deployed without any other Phase 2 sub-feature. If filtering (SF-38) is not yet available, labels still provide value as visible badges.

## User Stories

- As a Company Admin, I want to define a shared set of labels so that the team categorizes work consistently.
- As a Contributor, I want to tag a task with one or more labels so that it is easy to identify.

## Acceptance Criteria

- A label can be created with a name and a hex color.
- Duplicate label names within the same scope are rejected.
- A label can be applied to multiple tasks and a task can carry multiple labels.
- Deleting a label removes its associations without deleting the tasks.
- Color values are validated as valid hex.

## Data Requirements

- Label: id, name, color, company_id (nullable), user_id (nullable), created_at.
- Exactly one of company_id or user_id is set.
- Task_Label: task_id, label_id, with a unique constraint on the pair.

## Security and Isolation Requirements

- Labels are visible and usable only within their owning scope.
- Cross-scope label application is rejected.
- Authorization for label management follows the SF-13 matrix.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
