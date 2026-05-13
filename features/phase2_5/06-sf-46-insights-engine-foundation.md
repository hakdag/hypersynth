# SF-46 — Insights Engine Foundation

## Purpose

Provide a generic, deterministic mechanism for opening, tracking, and resolving rule-generated insights (alerts) about projects, tasks, users, and AI usage.

## Summary

This sub-feature introduces the `insight` entity and the evaluation loop that runs after each daily snapshot completes. It defines the lifecycle (open, kept open, auto-resolved) and the read API. No actual rules ship in this sub-feature; built-in rules are layered on top in SF-47. The engine is useful on its own because external or test rules can already produce insights through the same pipeline.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Add the `insight` entity (entity_type, entity_id, rule_id, severity, payload, opened_at, resolved_at, scoping fields).
- Define lifecycle:
  - Open when condition becomes true and no open insight for the same (rule_id, entity_type, entity_id) scope exists.
  - Auto-resolve when the condition no longer holds.
- Trigger evaluation after each daily snapshot completes (per SF-41).
- Expose a read API: list open insights by company, project, user; mark resolved insights as historical.
- Severity enum: Info, Warning, Critical.
- Insight descriptions are deterministic templates filled from the payload — no AI text.

## Out of Scope

- The built-in rule set (SF-47).
- Per-company configurable thresholds (SF-48).
- Insight notifications (email, in-app push).
- AI-generated explanations (Phase 3).

## Dependencies

- SF-13 User Roles and Authorization
- SF-14 Company Data Isolation
- SF-41 Metrics Snapshot Pipeline

## Independent Deployment Notes

Ships independently. With no rules registered yet, the engine simply produces no insights — the data model, lifecycle, and read API are still usable and testable.

## User Stories

- As a Company Admin, I want a unified inbox of operational alerts about my company.
- As a Project Manager, I want insights to auto-resolve when the underlying issue is gone so the list stays current.

## Acceptance Criteria

- Insights can be opened and auto-resolved by the evaluation loop.
- Each insight stores entity_type, entity_id, rule_id, severity, payload, opened_at, and (when resolved) resolved_at.
- Duplicate open insights for the same (rule_id, entity_type, entity_id) scope are prevented.
- The read API returns open and historical insights scoped to the requester.
- No AI is involved in producing insight text.

## Data Requirements

- New entity `insight` with fields from Phase 2.5 FRD section 11.2.
- Either `company_id` or `owner_user_id` must be set.

## Security and Isolation Requirements

- Insight reads are strictly scoped to the requester's company or personal account.
- Project-scoped insights are readable only by project members and Company Admin.
- Backend authorization is enforced on every read.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.
