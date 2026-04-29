# HyperSynth

**AI-driven project management — from requirements to execution, with human oversight at every step.**

HyperSynth is a multi-user SaaS platform that structures software projects as a hierarchy of **Projects → Features → Tasks** and uses AI to accelerate requirement refinement and task generation — while keeping humans in control through structured approval workflows.

---

## What It Does

Most project management tools help you track work. HyperSynth helps you **define** work.

- Write high-level project requirements in rich text
- Let AI enhance them, split them into features, and generate granular tasks
- Review every AI proposal through a diff-based approval screen before anything lands
- Execute with full task assignment, prioritization, dependencies, and collaboration
- Audit every action — who changed what, when, and why

---

## Feature Highlights

### AI-Assisted Planning
- Enhance project and feature requirements via AI
- Generate features automatically from project requirements
- Generate tasks from feature requirements
- All AI output goes through an approval workflow — nothing is applied without human review
- Full version history for every requirement with restore capability

### Structured Project Hierarchy
- **Projects** — top-level containers with rich-text requirements and status tracking
- **Features** — scoped units of work derived from project requirements
- **Tasks** — granular work items, created manually or AI-generated, with assignments, priorities, due dates, labels, and dependencies

### Real-World Execution
- Task assignments with project-membership enforcement
- Priority levels: Low, Medium, High, Critical
- Task statuses: Pending, In Progress, Blocked, In Review, Done, Cancelled
- Dependency graph with cycle detection — blocked tasks cannot be marked complete
- Comments with `@mention` support
- Full activity log on every entity change
- Project dashboard showing task breakdown, overdue items, and per-user workload

### SaaS-Ready Multi-Tenancy
- **Personal accounts** for individual users
- **Company accounts** with team management, email invitations, and role-based access control
- Role hierarchy: Company Admin → Project Manager → Contributor → Viewer
- Strict data isolation — no cross-company data leakage, enforced at the backend query level
- System Admin dashboard for platform-wide oversight

### Security & Compliance
- Passwords hashed with Argon2 or bcrypt
- AI API keys encrypted at rest, never logged or returned in full to the frontend
- Cryptographically secure invitation tokens with expiry and single-use enforcement
- Comprehensive audit logging for company events and system admin actions
- AI usage tracking per user and per company (tokens, estimated cost, errors)

---

## Role Model

### Company Roles

| Permission | Company Admin | Project Manager | Contributor | Viewer |
|---|:---:|:---:|:---:|:---:|
| Manage company profile | Yes | — | — | — |
| Invite users | Yes | Yes | — | — |
| Create / delete projects | Yes | Yes* | Yes* | — |
| Edit requirements | Yes | Yes | Yes | — |
| Generate / approve AI tasks | Yes | Yes | Yes | — |
| Manage AI settings | Yes | — | — | — |
| View projects | Yes | Yes | Yes | Yes |

### AI-Specific Roles (Phase 3)

| Role | Capability |
|---|---|
| **Enhancement Operator** | Trigger AI enhancement and generation jobs |
| **AI Approver** | Review, approve, reject, or edit AI proposals |

Users can hold multiple roles simultaneously. Enhancement Operators cannot approve their own proposals.

---

## Roadmap

| Phase | Status | Focus |
|---|---|---|
| **Phase 0** | Planned | Core MVP — user auth, project/feature/task management, AI integration |
| **Phase 1** | Planned | SaaS foundation — company accounts, RBAC, invitations, audit logs, AI usage tracking |
| **Phase 2** | Planned | Execution layer — assignments, priorities, due dates, dependencies, comments, dashboards |
| **Phase 3** | Planned | AI workflow engine — approval workflows, versioning, proposal review UI, AI feature generation |
| **Phase 4** | Planned | Enterprise features |

---

## Project Structure

```
hypersynth/
└── features/
    ├── phase0/   # PRD — core system requirements
    ├── phase1/   # FRD — SaaS foundation
    ├── phase2/   # FRD — project management enhancements
    ├── phase3/   # FRD — AI workflow engine
    └── phase4/   # FRD — enterprise features
```

---

## Data Model Overview

```
Company
└── Users (Company Admin, Project Manager, Contributor, Viewer)
    └── Projects
        ├── Features
        │   └── Tasks (with assignments, priorities, labels, dependencies, comments)
        ├── Documents (context for AI requests)
        └── AI Settings (encrypted API key, provider, model limits)

AI Proposals
└── RequirementVersions (original + all AI-generated, restorable)
    └── EnhancementProposals (pending → approved / rejected)

Audit Logs / AI Usage Records
```

---

## Key Design Principles

**Human oversight over AI automation.** Every AI-generated change is a proposal. Nothing overwrites human-written content without explicit approval by a designated AI Approver.

**Strict data isolation.** Multi-tenancy is enforced at the query level with `company_id` scoping on all company-owned entities — frontend hiding is not relied upon.

**Traceability by default.** Every assignment change, status transition, requirement edit, and AI action is recorded in activity logs and audit logs.

**Modular phased delivery.** Each phase builds on the previous without breaking it, enabling incremental deployment and validation.

---

## License

To be determined.
