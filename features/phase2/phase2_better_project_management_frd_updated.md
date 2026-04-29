# Feature Requirement Document (FRD)
## Phase 2 — Better Project Management (Detailed)
## AI-Driven Project Management System

**Date:** 2026-04-29

---

## 1. Purpose

This document defines Phase 2 of the system, which transforms the platform from a basic structured planning tool into a fully functional collaborative project management system.

The system must now support real-world execution workflows including ownership, prioritization, collaboration, and traceability.

This document is written to be **fully self-contained**, so an implementation agent with no prior knowledge can build the system from it.

---

## 2. System Behavior Overview

After Phase 2 implementation, the system must support:

- Assigning tasks to users
- Setting priorities and deadlines
- Categorizing tasks with labels
- Allowing collaboration through comments
- Tracking all changes (activity logs)
- Defining execution order using dependencies
- Filtering and sorting tasks efficiently
- Providing project-level insights

---

## 3. Task Management Enhancements

### 3.1 Task Assignment

Each task must optionally be assigned to a user.

#### Data:
- assignee_user_id (nullable)

#### Rules:
- Assignee must belong to same company or personal account
- If task belongs to project, assignee must be project member
- Task may remain unassigned

#### Behavior:
- Assign during create/edit
- Reassignment allowed
- Log all assignment changes

---

### 3.2 Task Priority

Each task must have a priority level.

#### Values:
- Low
- Medium (default)
- High
- Critical

#### Behavior:
- Used for sorting/filtering
- Changes logged in activity logs

---

### 3.3 Due Dates

#### Fields:
- due_date (nullable)
- due_time (nullable)

#### Behavior:
- Overdue = current_time > due_date
- Overdue is computed dynamically
- Filtering must support overdue tasks

---

### 3.4 Task Status Model

#### Values:
Pending, In Progress, Blocked, In Review, Done, Cancelled

#### Rules:
- Default: Pending
- Cannot mark Done if blocked by dependencies
- Status transitions logged

---

## 4. Labels

### 4.1 Definition

Labels are reusable tags.

Fields:
- id
- name
- color (hex)
- company_id OR user_id

Rules:
- Unique name per scope

---

### 4.2 Relationship

- Many-to-many with tasks

---

## 5. Comments

### 5.1 Model

Fields:
- id
- task_id
- user_id
- content
- created_at
- updated_at

### 5.2 Behavior

- Create/edit/delete comments
- Chronological order

---

### 5.3 Mentions

Syntax:
@username

Rules:
- Must belong to same scope
- Store parsed mentions

---

## 6. Activity Logs

### 6.1 Purpose

Track all entity changes.

### 6.2 Entities

- Project
- Feature
- Task

### 6.3 Fields

- entity_type
- entity_id
- user_id
- action_type
- old_value (JSON)
- new_value (JSON)
- timestamp

---

## 7. Dependencies

### 7.1 Model

Fields:
- task_id
- depends_on_task_id

### 7.2 Rules

- No self-dependency
- No cycles (must validate graph)
- Cannot complete if blocked

---

## 8. Filtering & Sorting

### Filtering:
- assignee
- status
- priority
- due_date
- labels

### Sorting:
- due_date
- priority
- created_at
- updated_at

---

## 9. Project Enhancements

### 9.1 Members

Explicit project membership table

### 9.2 Dashboard

Must show:
- tasks by status
- overdue tasks
- workload per user

---

## 10. Feature Enhancements

- Show grouped tasks
- Show progress %

progress = done / total

---

## 11. Data Model

### Task
- id
- assignee_user_id
- priority
- due_date
- status

### Label
- id
- name
- color

### Task_Label
- task_id
- label_id

### Comment
- id
- task_id
- user_id
- content

### Dependency
- task_id
- depends_on_task_id

### Activity_Log
- entity_type
- entity_id
- user_id
- action_type

---

## 12. Security

- Enforce company isolation
- Validate assignments and dependencies

---

## 13. Non-Functional

- Indexed queries
- Scalable to large task counts

---

## 14. Acceptance Criteria

- Tasks assignable
- Dependencies enforced
- Comments working
- Filtering functional
- Dashboard operational

---

## 15. Future

- Notifications
- Automation
- Time tracking
