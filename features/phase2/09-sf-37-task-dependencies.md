# SF-37 — Task Dependencies

## Purpose

Capture relationships between tasks so that execution order can be modeled and completion can be gated by prerequisites.

## Summary

This sub-feature introduces a directed dependency relation between tasks within the same scope, validates the graph (no self-loops, no cycles), and prevents a task from being marked Done while any of its prerequisites are unresolved. The validation enforces the soft guard introduced in SF-32.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add a Dependency entity with task_id and depends_on_task_id.
- Allow adding and removing dependencies between tasks in the same scope.
- Reject self-dependencies.
- Reject cycles using a graph check at write time.
- Reject Done transitions when any prerequisite is not in a terminal status.
- Surface incoming and outgoing dependencies on the task view.

## Out of Scope

- Dependency types (finish-to-start, start-to-start, etc.).
- Cross-scope dependencies.
- Automatic status propagation between dependent tasks.
- Critical-path visualization.

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation
- SF-32 Extended Task Status Model

## Independent Deployment Notes

Ships on top of SF-32. Activity log entries for dependency changes appear automatically if SF-36 is deployed; if not, the dependency feature still functions correctly.

## User Stories

- As a Project Manager, I want to declare that a task depends on another so that work happens in the correct order.
- As a Contributor, I want the system to prevent closing a task before its prerequisites are done so mistakes are caught early.

## Acceptance Criteria

- A dependency can be added between two distinct tasks in the same scope.
- Self-dependencies are rejected with a clear error.
- Adding an edge that would create a cycle is rejected with a clear error.
- Removing a dependency immediately lifts its constraints.
- A task cannot be moved to Done while any prerequisite is in a non-terminal status; the error explains which prerequisites are blocking.

## Data Requirements

- Dependency: task_id, depends_on_task_id, created_at.
- Unique constraint on (task_id, depends_on_task_id).
- Indexes to support both directions of traversal.

## Security and Isolation Requirements

- Dependency operations are scoped to the company or personal account; cross-scope edges are rejected.
- Edit rights for both endpoints are required to create or delete a dependency.
- Graph traversal must respect visibility rules.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
