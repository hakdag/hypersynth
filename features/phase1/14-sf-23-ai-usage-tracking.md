# SF-23 — AI Usage Tracking

## Purpose

Record every AI operation performed by users so that usage, cost, and errors can be observed at user, project, and company level.

## Summary

This sub-feature introduces the AI Usage entity and the recording mechanism. Whenever the system performs an AI operation, a usage record is created describing the operation, provider, model, token counts, estimated cost, and outcome.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add the AI Usage entity with all required fields.
- Hook into every AI operation defined for Phase 1: Enhance Project Requirements, Split Project Requirements into Features, Enhance Feature Requirements, Generate Tasks, Regenerate Tasks.
- Record both successful and failed operations, including error_code on failure.
- Compute estimated_cost from the provider/model and token counts using a simple, configurable mapping.
- Make usage records queryable by user, company, project, feature, and date range to enable later reporting and admin views.

## Out of Scope

- Enforcing the monthly token limit (could be a future enhancement that builds on this data).
- Real-time billing integration.
- End-user dashboards for usage (admin view is covered in SF-25; company/personal-facing UIs are future work).

## Dependencies

- SF-14 Company Data Isolation
- SF-21 Company AI Settings or SF-22 Personal AI Settings (whichever applies for the calling context)

## Independent Deployment Notes

Can be deployed independently and immediately starts adding observability value. The admin AI usage view (SF-25) consumes this data but is not required to ship together.

## User Stories

- As a System Admin, I want every AI operation recorded so that I can understand platform usage and cost.
- As a Company Admin, I want my company's AI usage to be measurable so that I can manage consumption later.

## Acceptance Criteria

- Every Phase 1 AI operation produces an AI Usage record.
- Records include provider, model, input_tokens, output_tokens, estimated_cost, status, and (on failure) error_code.
- Records are correctly attributed to user_id, and to company_id and project_id/feature_id where applicable.
- Failed AI operations still produce a usage record with status indicating failure.
- Usage records can be filtered and aggregated by user, company, project, and date range.

## Data Requirements

- AI Usage: id, company_id nullable, user_id, project_id nullable, feature_id nullable, operation_type, provider, model, input_tokens, output_tokens, estimated_cost, status, error_code nullable, created_at.

## Security and Isolation Requirements

- Usage records are subject to the same isolation rules as other company-scoped data (SF-14).
- Records must not include the AI API key or any prompt content beyond what is necessary for accounting.
- Personal users see only their own usage; company users see only their company's usage when usage views are exposed.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
