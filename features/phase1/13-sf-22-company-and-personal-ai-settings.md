# SF-22 — Project-Level AI Settings

## Purpose

Allow authorized users to configure and securely store AI settings on a per-project basis, for both company-owned and personal-owned projects.

## Summary

AI settings are bound to a project. Each project that uses AI features has its own settings record capturing a chosen provider, an encrypted API key, allowed models (from the catalog defined in SF-21), a monthly token limit, and a usage-tracking preference. API keys are encrypted at rest and never returned in full to the frontend.

Edit access depends on the project's ownership scope:

- For company-owned projects, only Company Admin and Project Manager users may view or modify the project's AI settings.
- For personal-owned projects, only the owning Personal Account user may view or modify the project's AI settings.

Provider and model definitions are owned by SF-21; this sub-feature only consumes them.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It depends on earlier foundation capabilities but does not require unrelated future features.

## Scope

- Provide a Project AI Settings screen reachable from a project's detail view.
- Restrict view and edit access to:
  - Company Admin and Project Manager for company-owned projects.
  - The owning user for personal-owned projects.
- Capture and store, per project: provider, encrypted_api_key, allowed_models, monthly_token_limit, usage_tracking_enabled.
- Encrypt the API key at rest using a server-side mechanism; never store plaintext.
- Display the API key as a masked value (for example, `sk-****abcd`) when shown back to the user.
- Allow authorized users to update or remove the stored key.
- Apply the project's AI settings when the system performs AI operations against that project (requirement enhancement, feature splitting, task generation, etc.).

## Out of Scope

- AI provider catalog and model registry (covered by SF-21).
- AI usage tracking and reporting (covered by SF-23).
- Company-level or user-level AI settings shared across projects.
- Switching between multiple keys per provider on a single project.
- AI provider selection at request time per user.

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation
- SF-21 AI Provider Settings

## Independent Deployment Notes

Can be deployed independently of AI usage tracking (SF-23). Existing project flows continue to work for projects without configured AI settings; AI features simply remain unavailable for those projects until settings are saved.

## User Stories

- As a Company Admin, I want to configure AI settings on a per-project basis so that each project uses the provider and key appropriate for its work.
- As a Project Manager, I want to configure AI settings for projects I manage so that I do not have to wait on a Company Admin to enable AI features.
- As a personal user, I want to configure AI settings for my own projects so that AI features work for my projects.
- As a security stakeholder, I want API keys to be encrypted and never shown in full so that they cannot be leaked.

## Acceptance Criteria

- A Company Admin or Project Manager can save AI settings for any project within their company.
- A Personal Account user can save AI settings for any project they own.
- Contributors, Viewers, and other non-authorized users cannot view or change a project's AI settings.
- Users from other companies cannot view or change a project's AI settings (per SF-14).
- API keys are encrypted at rest; database inspection does not reveal the plaintext key.
- The frontend never receives the full API key after it has been saved.
- AI operations performed against a project use that project's configured AI settings.

## Data Requirements

- Project AI Settings: id, project_id, provider, encrypted_api_key, allowed_models, monthly_token_limit, usage_tracking_enabled, created_at, updated_at.
- `project_id` is unique: at most one AI settings record per project.
- The owning company or owning user of the settings record is determined by following `project_id` to the project's ownership fields; no separate `company_id` or `user_id` is stored on the settings row.

## Security and Isolation Requirements

- The encryption mechanism must use a key managed outside the database.
- API keys must never be written to logs.
- Access is gated by project ownership and role:
  - Company-owned project: caller must be a member of the project's company with role Company Admin or Project Manager (per SF-13).
  - Personal-owned project: caller must be the project's owning user.
- All access checks must be enforced server-side, not only by hiding UI controls.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- Project-scoped data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
