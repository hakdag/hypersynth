# SF-17 — Project Membership

## Purpose

Provide project-level access control inside a company so that company users can be bound to specific projects.

## Summary

This sub-feature introduces the Project Membership entity and the screens and rules to manage it. It allows Company Admin and Project Manager users to add or remove members from a project and supports the project_id binding produced by invitation acceptance.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add a Project Membership entity linking a user and a project with a role.
- Provide a project members screen on the project detail view.
- Allow Company Admin and Project Manager to add or remove members from a project.
- Enforce that only members of a project may perform project-scoped actions (combined with SF-13 role rules).
- Honor project_id bindings created during invitation acceptance (SF-16).

## Out of Scope

- Project-level custom roles distinct from company roles.
- Cross-project bulk membership management.
- Inviting external users directly into a project (covered by SF-15/SF-16).

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation

## Independent Deployment Notes

Can be deployed independently. If invitations already store project_id (SF-15), this sub-feature activates that data. Until deployed, projects remain implicitly accessible to all company users at the level allowed by their role.

## User Stories

- As a Project Manager, I want to add and remove members from my project so that only the right people can see and edit it.
- As a Contributor, I want to access only the projects I am assigned to so that the workspace stays focused.

## Acceptance Criteria

- A Company Admin can add and remove members for any project in the company.
- A Project Manager can add and remove members for projects they manage.
- Project-scoped actions check membership in addition to company role.
- Adding a duplicate membership has no destructive effect.
- Removing a membership immediately revokes access for that user to that project.

## Data Requirements

- Project Membership: id, project_id, user_id, role, created_at.
- Unique constraint on (project_id, user_id).

## Security and Isolation Requirements

- Project membership operations are scoped to the company (per SF-14).
- A user cannot be added to a project from another company.
- Membership changes must be logged when audit logging is enabled (SF-24).

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
