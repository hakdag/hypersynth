# SF-25 — System Admin Dashboard: AI Usage Monitoring

## Purpose

Allow the System Admin to monitor AI usage across all companies and users to detect anomalies, high cost, and high failure rates.

## Summary

This sub-feature provides AI usage views inside the System Admin dashboard, including aggregations by company and user, token totals, estimated cost, failed-request counts, and a "high-usage companies" view.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Provide an AI usage overview page in the System Admin dashboard.
- Aggregate AI usage by company, user, provider, and model for a selectable date range.
- Show input tokens, output tokens, estimated cost, success count, failure count.
- Provide a high-usage companies view sortable by tokens or cost.
- Provide a failed AI requests view filterable by company, user, provider, and time range.

## Out of Scope

- Billing or invoicing.
- Setting or modifying limits from this screen (configuration is in SF-28).
- End-user (company-side) usage dashboards.

## Dependencies

- SF-18 System Admin Authentication
- SF-23 AI Usage Tracking

## Independent Deployment Notes

Can be deployed any time after AI usage tracking exists. Without SF-23, this sub-feature has no data to show and should not be enabled.

## User Stories

- As a System Admin, I want to see which companies use the most AI so that I can plan capacity and contact them.
- As a System Admin, I want to see failed AI requests so that I can detect provider or configuration issues early.

## Acceptance Criteria

- The System Admin can view aggregated AI usage by company and by user for a chosen date range.
- The high-usage view ranks companies by tokens or cost.
- Failed AI requests can be listed and filtered.
- All data shown is sourced from AI Usage records (SF-23).
- Only System Admin sessions can access these views.

## Data Requirements

- Reuses the AI Usage entity from SF-23.

## Security and Isolation Requirements

- Only System Admin sessions can access AI usage monitoring views.
- Views must not expose secrets such as API keys.
- Aggregations must not leak per-record content beyond the documented fields.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
