# SF-34 — Task Comments

## Purpose

Enable users to discuss work directly on a task so that context lives with the task itself.

## Summary

This sub-feature adds a Comment entity tied to a task and provides create, edit, and delete operations with chronological listing. Mentions are deliberately split into SF-35 so this base capability can ship first.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add a Comment entity with id, task_id, user_id, content, created_at, updated_at.
- Provide create, edit, and delete operations for comments.
- Restrict edit and delete to the comment’s author (with admin override per SF-13).
- List comments in chronological order on the task view.
- Treat content as plain text plus simple line breaks.

## Out of Scope

- Rich text or attachments.
- Threaded replies.
- Reactions.
- Mentions (covered by SF-35).
- Activity log entries for comment events (covered by SF-36).

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation
- SF-17 Project Membership

## Independent Deployment Notes

Comments ship without mentions or activity logs. When SF-35 and SF-36 are later deployed, they extend this feature without requiring rework here.

## User Stories

- As a Contributor, I want to leave a comment on a task so that questions and decisions are visible to the team.
- As an author, I want to edit or delete my own comment so that I can correct or retract it.

## Acceptance Criteria

- A user who can view a task can post a comment on it.
- A user can edit or delete their own comments; others cannot.
- Comments appear in chronological order on the task.
- Editing updates `updated_at` and is reflected on display.
- Deleting a comment removes it from views but leaves the task intact.

## Data Requirements

- Comment: id, task_id, user_id, content (non-empty), created_at, updated_at.
- Index on task_id for fast listing.

## Security and Isolation Requirements

- Comment access follows the same visibility rules as the underlying task.
- Cross-company comment access is rejected.
- Edit and delete are restricted to the author or a role permitted by SF-13.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
