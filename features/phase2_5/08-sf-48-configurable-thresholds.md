# SF-48 — Configurable Insight Thresholds

## Purpose

Let each company tune insight rule thresholds and health score weights to match their own delivery norms.

## Summary

This sub-feature introduces the `insight_rule_config` entity and the admin UI to view and edit threshold values per rule. It also exposes editing of the `health_score_config` weights (entity defined in SF-45). All edits are restricted to Company Admin, take effect on the next evaluation, and are recorded in the existing Phase 1 audit log.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add the `insight_rule_config` entity (rule_id, config JSON, scoping fields).
- Provide a Company Admin settings screen listing:
  - All registered insight rules with their current thresholds (default or company override)
  - Edit form for each threshold field
  - Edit form for Project Health Score weights (when SF-45 is deployed)
- Resolve effective config as: company override → default.
- Apply new config on the next snapshot / evaluation cycle.
- Record every config change in the Phase 1 audit log.

## Out of Scope

- Per-project or per-user overrides.
- Custom user-authored rules.
- Threshold editing for Personal Accounts (defaults only).
- Versioned config history beyond what the audit log provides.

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation
- SF-25 Audit Logging
- SF-46 Insights Engine Foundation
- SF-47 Built-in Insight Rules
- SF-45 Project Health Score (optional; health score weight editing only appears when SF-45 is deployed)

## Independent Deployment Notes

Ships independently after SF-46 and SF-47. Without it, default thresholds apply everywhere — companies simply cannot tune. With it, tuning becomes possible without changing any rule code.

## User Stories

- As a Company Admin, I want to raise or lower thresholds so that insights match how my company actually operates.
- As a Company Admin, I want my threshold changes to take effect quickly and to be auditable.

## Acceptance Criteria

- Company Admin can view and edit thresholds for every registered insight rule.
- Edits are validated against per-rule constraints (e.g. positive integers, percentages within range).
- Effective configuration resolves to the company override when present, otherwise the default.
- Threshold changes take effect on the next evaluation cycle.
- Every change is recorded in the Phase 1 audit log.
- Non-admin users cannot edit thresholds.
- Personal Accounts do not see this screen.

## Data Requirements

- New entity `insight_rule_config` with fields from Phase 2.5 FRD section 11.3.
- Reuses `health_score_config` (SF-45) where deployed.

## Security and Isolation Requirements

- Read and write of configuration is restricted to the requester's company.
- Only Company Admin may edit; backend enforces role check.
- Audit log entries include actor, action, before / after values.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
