# Feature Requirement Document (FRD)
## Phase 3 — AI Workflow Engine (Enhanced & Detailed)
## AI-Driven Project Management System

**Date:** 2026-04-29

---

# 1. Purpose

This document defines the enhanced Phase 3 AI Workflow Engine with **full approval workflows, role management, UI flows, and versioning strategy**.

This phase ensures:
- AI does not overwrite human work
- All AI changes are controlled and approved
- Users collaborate safely with AI
- System maintains full traceability and auditability

## 1.1 Relationship to Phase 2.5

Phase 2.5 (Insights & Performance) establishes a deterministic, rule-based insights panel and a KPI baseline. Phase 3 is the natural place to layer AI on top of that foundation:

- AI-generated explanations and recommended actions attached to Phase 2.5 insights
- AI summaries of project health, bottlenecks, and trends
- AI suggestions for unblocking dependency bottlenecks or rebalancing workload

These AI surfaces must follow the same approval, versioning, and role rules defined in this document. The deterministic insights and KPIs from Phase 2.5 remain authoritative; AI output augments them but does not replace them.

---

# 2. Key Additions in This Revision

This revision introduces:

- Approval role requirement for all AI enhancements
- New role: Enhancement Operator (renamed from EnhanceEnabled)
- Multi-role support per user
- Proposal review UI design
- Versioning with preservation of human-written content
- Version history UI
- AI feature generation from project requirements

---

# 3. Role Model Enhancements

## 3.1 Multi-Role Support

Users must support **multiple roles simultaneously**.

Update:

```text
UserRole (many-to-many)
- user_id
- role
```

A user may have:

```text
Company Admin + Enhancement Operator
Project Manager + Approval Role
Contributor + Enhancement Operator
```

---

## 3.2 New Roles

### Enhancement Operator (previously EnhanceEnabled)

Purpose:
- Allows user to trigger AI enhancement actions

Capabilities:
- See "Enhance" buttons in UI
- Trigger AI jobs

---

### Approval Role (recommended name: AI Approver)

Purpose:
- Allows user to approve/reject AI proposals

Capabilities:
- Access proposal review screen
- Approve/reject enhancements

---

## 3.3 Role Assignment

Role assignment must be done via existing **User Management screen**.

Enhancement:

- Add multi-role selection UI (checkbox-based)
- Roles can be assigned at:
  - Company level
  - Project level (optional extension)

No new screen required; extend existing role assignment interface.

---

# 4. AI Enhancement Scope

AI enhancement applies to:

- Project Requirements
- Feature Requirements
- Task Descriptions

ALL of these must follow approval workflow.

---

# 5. Requirement Enhancement Approval System (Extended)

## 5.1 Rule

```text
ANY AI-generated modification must be approved by a user with AI Approver role
```

---

## 5.2 Versioning Requirement

System must preserve:

- Original human-written version
- All AI-generated versions

After approval:

```text
Human version → remains as previous version
AI version → becomes active
```

---

## 5.3 Version History UI

Users must be able to:

- View all previous versions
- See who created each version
- See timestamps
- Restore previous versions

---

# 6. Proposal Management UI

## 6.1 Proposal List Screen

New screen required:

```text
"AI Proposals"
```

Accessible by:
- AI Approver role users

Displays:

- List of pending proposals
- Entity type (Project / Feature / Task)
- Created by
- Created date
- Status

---

## 6.2 Proposal Detail Screen

Must display:

### Left Panel:
- Current active version

### Right Panel:
- Proposed version

### Center:
- Diff viewer (highlight changes)

---

## 6.3 Actions

Buttons:

```text
Approve
Reject
Edit & Approve
```

---

## 6.4 Edit & Approve Flow

- User modifies proposed content
- System creates new version
- That version becomes active

---

## 6.5 Reject

 - User rejects with a reason which is a mandatory text field.

---

# 7. AI Feature Generation

## 7.1 Overview

AI must generate features from project requirements.

---

## 7.2 Flow

1. User clicks:
```text
"Generate Features from Project Requirements"
```

2. AI Job created

3. AI returns structured feature list

---

## 7.3 Feature Proposal Model

Features must be created as proposals:

- title
- description

---

## 7.4 Approval

Same approval workflow applies:

- AI Approver must approve features before creation

---

# 8. Task Enhancement

Task descriptions can be enhanced via AI.

Same rules apply:
- Proposal
- Approval
- Versioning

---

# 9. AI Job Enhancements

## 9.1 Job Types (Updated)

```text
EnhanceProjectRequirements
EnhanceFeatureRequirements
EnhanceTaskDescription
GenerateFeaturesFromProject
GenerateTasks
```

---

# 10. Approval Rules

- Only AI Approver role can approve/reject
- Enhancement Operator cannot approve unless also assigned Approver role
- No auto-approval allowed

---

# 11. Data Model

## Requirement

- id
- entity_type (project / feature)
- entity_id
- current_version_id

## RequirementVersion

- id
- requirement_id
- content
- version_number
- created_by (user_id or ai_job_id)
- is_active
- created_at

## RequirementEnhancement

- id
- requirement_id
- base_version_id
- proposed_version_id
- ai_job_id
- status
- created_by_user_id
- reviewed_by_user_id
- created_at
- reviewed_at

## EnhancementProposal

- id
- entity_type
- entity_id
- base_version_id
- proposed_version_id
- ai_job_id
- status
- created_by_user_id
- reviewed_by_user_id
- created_at
- reviewed_at

## UserRole (new)

- user_id
- role

---

# 12. Security Rules

- Only Enhancement Operator sees AI buttons
- Only AI Approver sees approval screens
- Users cannot approve their own proposals (optional rule)

---

# 13. Acceptance Criteria

System must:

- Require approval for all AI changes
- Support multiple roles per user
- Provide proposal review UI
- Preserve original versions
- Display version history
- Generate features from project requirements
- Restrict actions based on roles

---

# 14. Future Enhancements

- Role hierarchy
- Approval chains
- Multi-step approval
- AI confidence scoring
