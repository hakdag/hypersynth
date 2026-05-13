# SF-47 — Built-in Insight Rules

## Purpose

Provide the initial catalog of deterministic insight rules so that the Insights Engine produces actionable alerts out of the box.

## Summary

This sub-feature implements the built-in rules listed in Phase 2.5 FRD section 7.3 (blocked-task aging, critical-task aging, project velocity drop, forecast slip, overloaded user, unassigned backlog growth, dependency bottleneck, AI failure spike). Each rule evaluates against the latest snapshot and the previous snapshot, opens an `insight` when the condition is met, and lets the engine auto-resolve when the condition clears. Each rule produces a deterministic, template-filled description and a link to the affected entity.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Implement the following rules with documented defaults:
  - Blocked task aging (Warning / Critical)
  - Critical task aging (Critical)
  - Project velocity drop (Warning)
  - Forecast slip (Warning) — degrades gracefully if SF-49 is not yet deployed
  - Overloaded user (Warning)
  - Unassigned backlog growth (Info)
  - Dependency bottleneck (Warning)
  - AI failure spike (Warning)
- Each rule:
  - Reads only from `metric_snapshot` and existing entities.
  - Produces a deterministic description and a "next action" template-filled string.
  - Opens at most one open insight per (rule_id, entity_type, entity_id) at a time.
- Use built-in default thresholds; per-company tuning is added by SF-48.

## Out of Scope

- The insight engine itself (SF-46).
- Per-company configurable thresholds (SF-48).
- AI-generated explanations.

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation
- SF-41 Metrics Snapshot Pipeline
- SF-46 Insights Engine Foundation

## Independent Deployment Notes

Ships independently with the engine. Until SF-48 is deployed, all companies share the documented default thresholds. The Forecast slip rule is enabled only when its input (SF-49) is present; until then it is registered but inactive.

## User Stories

- As a Company Admin, I want to know when projects are slipping, users are overloaded, or AI is failing — without configuring anything.
- As a Project Manager, I want actionable, specific alerts that link directly to the affected entity.

## Acceptance Criteria

- All listed rules are registered with the engine.
- Each rule opens an insight when its condition is true and auto-resolves when the condition clears.
- Insight descriptions are template-filled and contain a link to the affected entity.
- Default thresholds are documented in code and in the API response payload.
- Rules requiring input from sub-features not yet deployed (e.g. Forecast slip) remain inactive without errors.

## Data Requirements

- No new persistent entities; uses `insight` (SF-46) and `metric_snapshot` (SF-41).

## Security and Isolation Requirements

- Insights produced by rules are scoped to the company / project / user they describe.
- Standard read scoping rules from SF-46 apply.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
