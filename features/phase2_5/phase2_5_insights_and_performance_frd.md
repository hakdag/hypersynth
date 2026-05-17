# Feature Requirement Document (FRD)
## Phase 2.5 — Insights & Performance
## AI-Driven Project Management System

**Date:** 2026-05-13

---

## 1. Purpose

This document defines Phase 2.5 of the system, an intermediate phase positioned between Phase 2 (Better Project Management) and Phase 3 (AI Workflow Engine).

Phase 2.5 transforms the platform from a collaborative project management tool into a system that **measures delivery, surfaces bottlenecks, and gives company management actionable insight** into how their projects, teams, and AI usage are performing.

The phase intentionally uses **deterministic, rule-based analytics**. No machine learning or AI-driven prediction is introduced here. That layer arrives in Phase 3 and benefits from the measurement foundation established here.

This document is written to be fully self-contained.

---

## 2. Rationale

By the end of Phase 2, the system already collects the raw signals required for analytics:

- Task lifecycle and status transitions (Phase 2 activity logs)
- Assignments, priorities, due dates, dependencies (Phase 2 task model)
- Project / feature / task hierarchy and progress (Phase 2)
- Comments and mentions (Phase 2)
- AI usage records: tokens, cost, operation type, status (Phase 1)
- Audit logs (Phase 1)
- Company / user / role structure (Phase 1)

Phase 2.5 turns these signals into management-grade indicators **before** Phase 3 introduces AI-generated content. This ordering ensures:

- A measurable baseline exists before AI changes the workflow
- Phase 3 AI features can be evaluated against pre-AI KPIs
- Bottlenecks become input signals for Phase 3 AI recommendations
- Management can justify and prioritize AI spend with real numbers

---

## 3. Scope

### 3.1 Included

- Metrics pipeline computed from existing Phase 1 / Phase 2 data
- Three dashboards: Company, Project, Personal
- KPI catalog with deterministic definitions
- Rule-based insights / alerts panel
- Configurable thresholds per company
- CSV / JSON export of metrics

### 3.2 Excluded

- Reports (weekly / monthly digests) — deferred
- Email delivery of any kind — deferred
- AI-driven predictions, recommendations, or natural-language summaries — Phase 3
- Forecasting beyond simple linear projection — Phase 3
- Custom KPI / dashboard builder — Phase 4
- Cross-company benchmarking — Phase 4
- Time tracking — out of scope (remains Phase 2 future)

### 3.3 Role and permission strategy

Phase 2.5 does **not** introduce new roles. It reuses the roles defined in Phase 1:

| Role | Access |
|---|---|
| Company Admin | All dashboards (Company, Project, Personal) |
| Project Manager | Project dashboards for projects they manage, Personal dashboard |
| Contributor | Personal dashboard, Project dashboards (read-only) for their projects |
| Viewer | Project dashboards (read-only) for their projects, Personal dashboard |
| System Admin | No company dashboards. Existing Phase 1 admin tools remain unchanged. |

Personal Accounts see the Project dashboard for their own projects and the Personal dashboard.

---

## 4. Audience and Questions Answered

Phase 2.5 is designed around the questions management actually asks.

| Question | Surface |
|---|---|
| Are we on track? | Delivery & Progress metrics |
| Where are we stuck? | Bottlenecks & Flow metrics |
| Is the team balanced? | Capacity & Workload metrics |
| Is our priority discipline working? | Priority & Risk metrics |
| Is AI worth it? | AI ROI metrics |
| Are we improving? | Trend views |
| What should I do right now? | Insights / alerts panel |

---

## 5. KPI Catalog

All KPIs must be deterministic and explainable. Every KPI must expose its formula in the UI (tooltip or info icon).

### 5.1 Delivery and Progress

| KPI | Definition | Source |
|---|---|---|
| Throughput | Tasks moved to Done per week | activity_log |
| On-time completion rate | done_before_due / done_total | task |
| Overdue rate | overdue_open / total_open | task |
| Feature progress % | done_tasks / total_tasks per feature | task |
| Scope change rate | Tasks added after project start_date / total tasks | task + activity_log |

### 5.2 Bottlenecks and Flow

| KPI | Definition | Source |
|---|---|---|
| Cycle time (median) | done_at − in_progress_at | activity_log |
| Lead time (median) | done_at − created_at | task + activity_log |
| Time-in-status | Duration a task spends in each status | activity_log |
| Blocked aging (p90) | 90th percentile of time spent in Blocked status | activity_log |
| Reopen rate | reopened_count / done_total | activity_log |
| Dependency bottleneck score | Number of open tasks blocked by a given task | dependency |

### 5.3 Capacity and Workload

| KPI | Definition | Source |
|---|---|---|
| WIP per user | Count of In Progress tasks per assignee | task |
| Workload index | Weighted sum of open tasks by priority per assignee | task |
| Unassigned task backlog | Count of open tasks with no assignee | task |
| Active contributors | Distinct users with task or comment activity in last N days | activity_log + comment |
| Assignment fairness | Standard deviation of workload index across project members | task |

### 5.4 Priority and Risk

| KPI | Definition | Source |
|---|---|---|
| Critical/High aging | Age of oldest open Critical / High tasks | task |
| Priority mix | Open Critical count vs. team size | task |
| Due-date discipline | done_before_due / done_total (same as on-time) | task |
| Project health score | Composite of overdue, blocked, slip, WIP (see 5.7) | computed |

### 5.5 AI ROI

| KPI | Definition | Source |
|---|---|---|
| AI utilization | AI operations per active user | ai_usage |
| AI cost per shipped task | Sum(estimated_cost) / count(done tasks) over period | ai_usage + task |
| AI task survival rate | ai_tasks_done / ai_tasks_created | task |
| AI error rate | failed_ai_ops / total_ai_ops | ai_usage |

### 5.6 Trends

Every numeric KPI must support:

- 7-day, 30-day, and 90-day windows
- Period-over-period delta (current vs previous equal-length window)
- Sparkline visualization

### 5.7 Project Health Score

A composite indicator on a 0–100 scale:

```text
health_score = 100
  − overdue_penalty
  − blocked_penalty
  − slip_penalty
  − wip_penalty
```

Each penalty is bounded and configured by company thresholds (see Section 8). The exact weights must be stored, not hard-coded, so they can be tuned later.

---

## 6. Dashboards

### 6.1 Company Dashboard

Audience: Company Admin (primary), Project Manager (read).

Sections:

- Portfolio overview: project count by status, average health score
- Top at-risk projects (by health score)
- Throughput trend (company-wide)
- AI ROI summary
- Workload heatmap across users
- Top dependency bottlenecks across projects

### 6.2 Project Dashboard

Audience: Project Manager, Company Admin, project members.

This extends the dashboard defined in Phase 2 (which already showed tasks by status, overdue, workload).

Adds:

- Project health score with breakdown
- Burndown / burnup chart
- Cycle time and lead time distributions
- Blocked tasks aging list
- Dependency bottleneck list
- AI usage and AI task survival rate for the project
- Trend views for throughput, on-time rate, overdue rate
- Linear forecast: projected completion date based on current velocity

### 6.3 Personal Dashboard

Audience: Every authenticated non-admin user.

Sections:

- My open tasks grouped by status and priority
- My overdue tasks
- Tasks blocking me / tasks I am blocking
- My workload index vs team average
- My throughput trend (last 4 weeks)
- Mentions and unread comment activity

---

## 7. Insights and Alerts

Phase 2.5 introduces a rule-based **Insights panel** visible on the Company and Project dashboards.

### 7.1 Behavior

- Insights are generated by deterministic rules evaluated on a schedule (see Section 9)
- Each insight has: entity_type, entity_id, rule_id, severity, opened_at, resolved_at (nullable), payload
- An insight auto-resolves when the underlying condition no longer holds
- No AI-generated text. Phase 3 may later attach AI explanations on top.

### 7.2 Severity levels

```text
Info
Warning
Critical
```

### 7.3 Built-in rules (initial set)

| Rule | Trigger condition | Severity |
|---|---|---|
| Blocked task aging | Task in Blocked status > threshold days | Warning / Critical |
| Critical task aging | Critical priority open > threshold days | Critical |
| Project velocity drop | Rolling 4-week throughput drops ≥ X% vs prior 4 weeks | Warning |
| Forecast slip | Forecast completion date later than previous snapshot by ≥ N days | Warning |
| Overloaded user | Workload index ≥ X above team average | Warning |
| Unassigned backlog growth | Unassigned open tasks grew by ≥ X% in last period | Info |
| Dependency bottleneck | Single task blocks ≥ N other open tasks | Warning |
| AI failure spike | AI error rate ≥ threshold over period | Warning |

All thresholds are configurable per company (see Section 8).

### 7.4 Actionable surface

Each insight must include:

- A short, deterministic description (template-filled, not AI)
- A link to the affected entity (project, task, user)
- Suggested next action (also template-based)

Example:

```text
Project "Apollo" velocity dropped 42% vs the prior 4 weeks.
Open the project dashboard to review WIP and blocked tasks.
```

---

## 8. Configurable Thresholds

Each Company stores its own configuration. Defaults are provided.

Fields (per company):

- blocked_task_warning_days
- blocked_task_critical_days
- critical_task_aging_days
- velocity_drop_warning_pct
- forecast_slip_warning_days
- workload_overload_delta
- unassigned_backlog_growth_pct
- dependency_bottleneck_min_blocked
- ai_failure_rate_warning_pct
- health_score_weights (object: overdue, blocked, slip, wip)

Only Company Admin may edit thresholds.

Personal Accounts use built-in defaults; threshold editing is not exposed.

---

## 9. Metrics Pipeline

### 9.1 Computation model

- Aggregations are computed on a schedule (daily snapshots) and on-demand for the current period
- Heavy queries must not run inline in dashboard requests
- Dashboards read from pre-aggregated tables wherever possible

### 9.2 Snapshotting

Daily snapshots must capture per-project and per-company values for all KPIs in Section 5.

This enables:

- Trend lines without re-scanning historical activity logs
- Period-over-period deltas
- Forecast slip detection (comparing today's forecast to yesterday's snapshot)

### 9.3 Real-time vs snapshot

| Surface | Source |
|---|---|
| Current open / overdue counts | Live query |
| Throughput, cycle/lead time, trends | Snapshots |
| Health score | Snapshots + live overlay for today |
| Insights | Evaluated on snapshot completion |

### 9.4 Insight evaluation

After each snapshot completes, the system evaluates all insight rules against the new snapshot and the previous snapshot. Insights are opened, kept open, or resolved accordingly.

---

## 10. Export

### 10.1 Requirements

- CSV export of any KPI table visible on a dashboard
- JSON export of the same data for programmatic consumers
- Export must respect the scope and permissions of the requesting user

### 10.2 Out of scope

- Scheduled exports
- Export delivery via email
- Export to external storage (S3, etc.)

These are reserved for Phase 4 data governance.

---

## 11. Data Model

Phase 2.5 adds the following tables. No Phase 1 / Phase 2 tables are modified.

### 11.1 metric_snapshot

Stores daily aggregated KPI values.

Fields:

- id
- company_id nullable
- owner_user_id nullable
- project_id nullable
- scope (Company / Project / User)
- metric_key
- metric_value (numeric)
- metric_payload (JSON, optional)
- snapshot_date
- created_at

Constraint: either company_id or owner_user_id must be set.

### 11.2 insight

Stores rule-generated alerts.

Fields:

- id
- company_id nullable
- owner_user_id nullable
- project_id nullable
- entity_type
- entity_id nullable
- rule_id
- severity
- payload (JSON)
- opened_at
- resolved_at nullable
- created_at
- updated_at

### 11.3 insight_rule_config

Stores per-company threshold configuration.

Fields:

- id
- company_id nullable
- owner_user_id nullable
- rule_id
- config (JSON)
- created_at
- updated_at

Constraint: either company_id or owner_user_id must be set.

### 11.4 health_score_config

Stores composite health score weights per company.

Fields:

- id
- company_id nullable
- owner_user_id nullable
- weights (JSON: overdue, blocked, slip, wip)
- created_at
- updated_at

Constraint: either company_id or owner_user_id must be set.

---

## 12. Security and Isolation

- All Phase 2.5 entities must be company-scoped (company_id) or personal-scoped (owner_user_id), following the Phase 1 isolation rule
- Backend authorization must be enforced on every read; frontend filtering is not sufficient
- A user must never see metrics, insights, or thresholds belonging to another company or personal account
- Project-scoped metrics must be readable only by project members and Company Admin
- Personal dashboard data must be readable only by the owning user
- Export requests must reapply the same authorization checks at request time

---

## 13. Non-Functional Requirements

### 13.1 Performance

- Dashboard load must be served primarily from snapshot tables
- Live queries on the dashboard must not scan full activity_log history
- Common metric reads must complete within typical interactive response time even at large task counts

### 13.2 Scalability

- Snapshot job must remain bounded as projects and tasks grow (per-project aggregation, not global scans)
- Insight evaluation must run incrementally against the latest snapshot

### 13.3 Maintainability

- KPI definitions live in one place (a metric catalog module) so formulas can be audited
- Insight rules are pluggable and versioned by rule_id

### 13.4 Auditability

- KPI snapshots are immutable once written
- Threshold changes must be recorded via the existing Phase 1 audit log

---

## 14. Acceptance Criteria

The phase is complete when:

- Daily snapshots are produced for every active company and project
- Company Admin can open a Company Dashboard with portfolio health, throughput trend, AI ROI, workload heatmap, and top at-risk projects
- Project Manager can open a Project Dashboard with health score, burndown, cycle/lead time, blocked aging, dependency bottlenecks, AI ROI, trends, and forecast
- Every user can open a Personal Dashboard with their open work, overdue, blockers, workload, throughput trend, and mentions
- All KPIs in Section 5 are computed using their documented formulas
- The Insights panel displays open insights with severity, description, and action link
- Built-in rules in Section 7.3 fire and auto-resolve correctly
- Company Admin can edit thresholds in Section 8; changes take effect on the next evaluation
- CSV and JSON export work for every dashboard KPI table
- Company isolation is enforced on every Phase 2.5 read path
- No Phase 1 or Phase 2 table is modified

---

## 15. Constraints

- No reports (weekly / monthly digests) in this phase
- No email delivery in this phase
- No new roles; reuse Phase 1 roles
- No AI-generated content; insight text is template-based
- No forecasting model beyond simple linear projection from current velocity
- No cross-company analytics

---

## 16. Future Enhancements

- Reports and digests (weekly / monthly)
- Email and in-app notifications for insights
- AI-generated explanations and recommended actions on top of insights (Phase 3 handoff)
- Confidence-scored forecasts (Phase 3)
- Custom KPI builder and saved views (Phase 4)
- Cross-company benchmarking for platform operators (Phase 4)
- Scheduled exports and external storage delivery (Phase 4)
- Time tracking integration
