# SF-13/SF-14 Project Reference

Use this file with `SKILL.md` for concrete checks against this repository.

## Canonical Source Documents

- `features/phase1/04-sf-13-user-roles-and-authorization.md`
- `features/phase1/05-sf-14-company-data-isolation.md`
- `features/phase1/phase1_saas_foundation_frd_updated.md`

## Current Backend Anchors

- Role and account typing:
  - `src/backend/src/types/company_role.rs`
  - `src/backend/src/types/account_type.rs`
- Auth/session extraction:
  - `src/backend/src/auth_route.rs`
  - `src/backend/src/types/session_user.rs`
- Centralized role gating:
  - `src/backend/src/authorization.rs`
- Main CRUD and AI route surface:
  - `src/backend/src/project_route.rs`
- Data access helpers that also need tenant checks:
  - `src/backend/src/document_context_service.rs`
  - `src/backend/src/project_api_key_service.rs`

## Query Review Patterns

When reviewing SQL in routes/services:

1. Identify tenant scope from session:
   - Personal scope expects `owner_user_id` predicate.
   - Company scope expects `company_id` predicate.

2. Confirm every read query includes tenant ownership:
   - `SELECT ... WHERE ... tenant predicate ...`
   - Includes nested resources (`feature`, `task`, `document`) through project joins.

3. Confirm every write query includes tenant ownership:
   - `INSERT` validates parent project scope before insert.
   - `UPDATE` and `DELETE` include tenant predicate in the modified row set.

4. Confirm bulk and AI flows are scoped:
   - Document context lookup and AI key runtime lookup must be tenant-filtered.

## Error-Handling Expectations

- Cross-tenant access should not leak record existence.
- For entity fetch/update paths, prefer neutral not-found style responses for unauthorized tenant rows.
- Role denial remains explicit (`403`) when action is disallowed by role policy.

## Migration/Schema Checks

When schema changes are part of SF-13/SF-14:

- Ownership model is explicit and enforceable in DB constraints.
- Nullability and CHECK constraints match account model.
- Indexes exist for tenant predicates used in route SQL.
- Backfill logic preserves ownership and does not create ambiguous tenant rows.

## Minimum Regression Scenarios

At least one end-to-end isolation check should prove:

1. User from Company A cannot access Company B project by ID.
2. Personal user cannot access company project.
3. List endpoints return only tenant-scoped data.

## Suggested Review Output

Keep output compact and ordered:

1. Critical finding(s)
2. Missing tenant/role guardrails
3. Required code changes
4. Optional hardening
5. Pass/fail
