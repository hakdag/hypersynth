# SF-11 — Company Account Registration

## Purpose

Allow a user to register a new Company Account by capturing required company information together with the first user, who becomes the Company Admin.

## Summary

This sub-feature implements the company registration flow. It collects required company fields, the first user's identity fields, creates a Company record and a Company Admin user linked to it, and redirects the new admin to the company workspace.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Provide a company registration screen following the account type selection.
- Capture required company fields: company name, company email, country, timezone.
- Capture optional company fields where useful: legal name, website, industry, company size, phone, billing email, address, tax/VAT number.
- Capture first user fields: full name, email, username, password, password confirmation.
- Validate required fields and email format.
- Prevent duplicate company email and duplicate user email at registration time.
- Store the password only in securely hashed form.
- Create a Company record with status Active.
- Create a User record with account_type Company and role Company Admin, linked to the new company.
- Redirect the new Company Admin to the company workspace landing area after registration.

## Out of Scope

- Inviting other users to the company.
- Editing or managing the company profile after registration.
- Billing collection.
- Email verification.
- Company verification flow.

## Dependencies

- SF-10 Account Type Selection at Registration

## Independent Deployment Notes

Can be deployed independently from team invitations. After registration, a Company Admin can use the system as a single-user workspace until invitation features are added later.

## User Stories

- As a new company user, I want to register my company and create my admin account in one flow so that I can start using the platform on behalf of my company.
- As a Company Admin, I want to land on a company workspace after registration so that I have a clear starting point.

## Acceptance Criteria

- A user can complete company registration with all required fields and a valid first user.
- Registration fails when any required company or user field is missing or invalid.
- Registration fails when the company email or user email is already used.
- A Company record is created with status Active.
- The first user is created with account_type Company and role Company Admin and is linked to the new company.
- Stored passwords are hashed; no plaintext password is persisted.
- The new Company Admin is redirected to the company workspace after success.

## Data Requirements

- Company: id, name, legal_name nullable, company_email, website nullable, industry nullable, company_size nullable, country, timezone, phone nullable, billing_email nullable, address nullable, tax_vat_number nullable, status, created_at, updated_at.
- User: id, account_type, company_id, full_name, display_name nullable, email, username, password_hash, role, status, timezone nullable, created_at, updated_at.
- Company email is unique. User email is unique. Username is unique.

## Security and Isolation Requirements

- Passwords must never be stored or displayed in plaintext.
- The first user must be created atomically with the Company; partial creation must not leave the system in an inconsistent state.
- The created company must be isolated from any other company from creation onward.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
