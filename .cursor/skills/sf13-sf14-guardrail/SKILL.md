---
name: sf13-sf14-guardrail
description: Verify backend changes preserve Phase 1 user roles/authorization (SF-13) and company data isolation (SF-14). Use when editing auth, routes, SQL queries, migrations, session/tenant logic, or any company/personal data access flow.
---

# SF-13/SF-14 Guardrail

## Purpose

Keep development aligned with:

- SF-13: user roles and authorization
- SF-14: company data isolation

Apply this skill whenever a task touches authentication, authorization, tenant resolution, route handlers, migrations, query predicates, or entity ownership fields.

## Core Rules To Enforce

1. Authorization is enforced on the backend, not only hidden in frontend UI.
2. Data access is tenant-scoped for every read/write path.
3. URL identifiers are never trusted without ownership/tenant checks.
4. Company users must not access other companies' records.
5. Personal users must not access company-scoped records.
6. Role checks are centralized and consistently applied.
7. Cross-tenant misses should not leak existence details.

## Review Workflow

Copy this checklist and track progress:

```text
SF13/SF14 Verification
- [ ] Identify affected entities/routes/services
- [ ] Verify tenant-resolution source (session -> active tenant scope)
- [ ] Verify query predicates on all reads
- [ ] Verify query predicates on all writes
- [ ] Verify role gate placement for protected actions
- [ ] Verify migration constraints/indexes preserve isolation invariants
- [ ] Verify error behavior does not leak cross-tenant existence
- [ ] Verify regression coverage for cross-tenant access denial
```

## What To Inspect

- **Session and identity**
  - Session payload includes account type and company context when relevant.
  - Tenant scope is derived once and reused, not re-invented per route.

- **Query enforcement**
  - Reads include tenant/owner predicates.
  - Writes include tenant/owner predicates in `INSERT`, `UPDATE`, and `DELETE` paths.
  - Join chains (`projects -> features -> tasks -> documents`) retain tenant constraints.
  - Bulk/search/AI-related operations cannot bypass tenant filters.

- **Role enforcement**
  - Protected actions call centralized role checks.
  - Company role permissions match Phase 1 FRD matrix.
  - Missing or invalid role/company context fails closed.

- **Schema integrity**
  - Ownership fields are explicit and constrained.
  - Constraints enforce valid tenant shape for records.
  - Indexes support tenant predicates used by routes.

## Output Format

When this skill runs, provide findings in this order:

1. Critical risks/regressions (if any)
2. Gaps vs SF-13/SF-14 requirements
3. Required fixes
4. Optional hardening suggestions
5. Final pass/fail statement

If no issues are found, say so clearly and mention remaining risk or test gaps.

## Minimal Acceptance Gate

Do not mark work complete unless all are true:

- Backend role checks exist for protected company actions.
- Backend tenant filtering exists for all relevant entity operations.
- Cross-tenant access attempt is denied without data leakage.
- A regression test (or equivalent verification) confirms isolation.

## Future Expansion

When the project adds new phase capabilities (invitations, memberships, AI usage, audit logs, admin flows), extend this skill by adding:

- New entity-specific guardrails
- New checklist items
- Required regression scenarios
