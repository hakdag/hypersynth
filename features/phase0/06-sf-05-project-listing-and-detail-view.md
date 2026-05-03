# SF-05 — Project Listing and Detail View

## Purpose

Allow a user to see their own projects and open a selected project.

## Summary

This sub-feature provides read access to owned projects. It can be deployed independently after project creation.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Display a list of projects owned by the authenticated user.
- Show project name and status in the list.
- Provide an empty state when the user has no projects.
- Allow the user to open a project detail view.
- Display project name, requirements, status, and AI API key presence indicator.
- Do not reveal the raw AI API key unless explicitly required by a later secure edit flow.

## Out of Scope

- Project creation.
- Project editing.
- Feature, task, document, and AI workflows.
- Cross-user project discovery.

## Dependencies

- SF-04 Project Creation

## Independent Deployment Notes

Can be deployed as a read-only project browser. It remains useful even before feature and task management exist.

## User Stories

- As a user, I want to see my project list so that I can choose what to work on.
- As a user, I want to open a project detail page so that I can inspect its current requirements and status.

## Acceptance Criteria

- The project list shows only projects owned by the authenticated user.
- A user with no projects sees an empty state.
- Opening a project displays its details.
- Opening another user’s project by direct identifier is blocked.
- AI API key is not displayed as plaintext in read-only detail mode.

## Data Requirements

- Reads Project records scoped by user_id.
- Shows status values: Pending, In Progress, Done.

## Security and Isolation Requirements

- Project list and detail access must enforce workspace isolation.
- Sensitive project configuration must not be exposed unnecessarily.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.

