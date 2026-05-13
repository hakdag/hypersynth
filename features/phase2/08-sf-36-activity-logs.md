# SF-36 — Activity Logs

## Purpose

Record changes to projects, features, and tasks so that history is traceable and auditable.

## Summary

This sub-feature introduces a single activity log stream that captures structured before/after values for create, edit, and delete actions on Project, Feature, and Task entities. It consumes the change events produced by other Phase 2 sub-features and presents a chronological history on the corresponding detail views.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add an Activity Log entity with entity_type, entity_id, user_id, action_type, old_value (JSON), new_value (JSON), timestamp.
- Capture create, edit, and delete actions for Project, Feature, and Task.
- Consume change events emitted by SF-29, SF-30, SF-31, SF-32, and SF-37 when those features are present; record only the actions that exist at deploy time.
- Provide a chronological history view per entity.
- Provide read access scoped by SF-13 and SF-14 rules.

## Out of Scope

- A unified global audit log across all entities (covered separately by SF-24 audit logging).
- Activity log filtering UI beyond entity-scoped chronological listing.
- Comment activity (treated as comments themselves are the record).
- Reverting changes from the activity log.

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation

## Independent Deployment Notes

Can be deployed before or after individual Phase 2 sub-features. When deployed, it begins consuming whatever change events already exist; new events automatically appear as their producing sub-features ship.

## User Stories

- As a Project Manager, I want to see who changed what on a task so that I can understand the history of work.
- As a Company Admin, I want a reliable change history per entity so that disputes can be investigated.

## Acceptance Criteria

- Create, edit, and delete actions on Project, Feature, and Task generate log entries.
- Each entry records the acting user, the action type, and the prior and new values when applicable.
- Entries are visible to users authorized to view the underlying entity.
- The log is append-only; entries cannot be edited or deleted through the application.

## Data Requirements

- Activity_Log: id, entity_type, entity_id, user_id, action_type, old_value, new_value, timestamp.
- Indexes on (entity_type, entity_id, timestamp).

## Security and Isolation Requirements

- Log entries are visible only to users authorized for the underlying entity.
- Cross-company log queries are rejected.
- Persisted old/new values must respect the same isolation rules as the source entity.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
