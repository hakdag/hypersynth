# SF-35 — Comment Mentions

## Purpose

Allow users to reference teammates inside comments so that attention can be directed to specific people.

## Summary

This sub-feature parses `@username` tokens in comment content, validates that the mentioned user belongs to the comment’s scope, and persists the resolved mentions alongside the comment. The mention list is exposed on the comment view; notification delivery is intentionally out of scope and reserved for a future sub-feature.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Parse `@username` tokens during comment create and edit.
- Resolve each mention to a user within the comment’s scope (same company or project membership where applicable).
- Persist the resolved mentions as structured data on the comment.
- Expose mentions in the comment response so the frontend can render them as links/badges.
- Silently drop unresolved or out-of-scope mentions (or mark them as plain text); do not fail comment submission.

## Out of Scope

- Notifying mentioned users.
- Group mentions (e.g., @team).
- Email or push delivery of mentions.
- Autocomplete suggestions (frontend-only concern handled separately).

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation
- SF-34 Task Comments

## Independent Deployment Notes

Ships on top of SF-34. Notifications are not required for this feature to be valuable; mentions provide visible attribution and an integration point for a future notifications sub-feature.

## User Stories

- As a Contributor, I want to mention a teammate in a comment so that the comment clearly points to them.
- As a reader, I want to see which teammates were mentioned in a comment so that I understand who is being addressed.

## Acceptance Criteria

- Comments containing `@username` produce a parsed mention list when the username resolves within scope.
- Out-of-scope or unknown usernames do not block comment submission.
- Editing a comment re-parses mentions consistently.
- The mention list is returned with the comment payload.

## Data Requirements

- Mentions stored as a list of resolved user_ids associated with the comment (separate table or JSON column, as appropriate).
- Persisted mentions never include users outside the comment’s scope.

## Security and Isolation Requirements

- Mention resolution must respect company and project visibility.
- A mention must not leak the existence of a user outside the requester’s scope; unknown handles resolve to nothing.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
