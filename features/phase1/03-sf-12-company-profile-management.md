# SF-12 — Company Profile Management

## Purpose

Allow Company Admin users to view and update their company profile after registration.

## Summary

This sub-feature provides a company profile screen where the Company Admin can review and edit the company information captured at registration. Other roles may view a read-only subset.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Provide a company profile screen accessible from the company workspace.
- Display all company fields, with required fields clearly marked.
- Allow the Company Admin to edit editable fields and save changes.
- Validate required fields and field formats on save.
- Restrict editing to Company Admin; other roles see a read-only view if shown at all.
- Update the updated_at timestamp on each successful change.

## Out of Scope

- Changing company status (Active/Disabled). That belongs to System Admin tooling.
- Deleting the company.
- Billing-specific fields beyond what was captured at registration.
- Multi-company switching.

## Dependencies

- SF-11 Company Account Registration
- SF-13 User Roles and Authorization (for the Company Admin restriction)

## Independent Deployment Notes

Can be deployed as soon as a company exists. Without role enforcement (SF-13), the screen can still be delivered to the registered Company Admin only by relying on the role assigned at registration.

## User Stories

- As a Company Admin, I want to update company information so that the workspace reflects current details.
- As a non-admin company user, I do not want to be able to change company settings so that company configuration stays controlled.

## Acceptance Criteria

- A Company Admin can open the company profile screen and see current values.
- A Company Admin can edit and save changes; required fields are validated.
- A non-admin company user cannot edit the company profile.
- Successful save updates the stored company record and updated_at timestamp.
- Validation errors are clearly shown without losing user input.

## Data Requirements

- Company entity from SF-11 is reused; no new entity is added.

## Security and Isolation Requirements

- The screen must only show the requesting user's own company.
- Authorization must be enforced on the backend; hiding controls on the frontend is not enough.
- Sensitive fields (e.g., billing email) must follow the same protection as other company data.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
