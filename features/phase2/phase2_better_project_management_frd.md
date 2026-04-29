# Feature Requirement Document (FRD)
## Phase 2 — Better Project Management
## AI-Driven Project Management System

**Date:** 2026-04-29

---

## 1. Purpose

This document defines Phase 2 enhancements focused on evolving the system from a structured planning tool into a fully capable project management system.

Phase 2 introduces:
- Task assignment and ownership
- Due dates and scheduling
- Priorities and labels
- Comments and collaboration
- Activity tracking
- Task dependencies
- Enhanced statuses

---

## 2. Scope

Includes:
- Task assignment
- Priorities
- Due dates
- Labels
- Comments
- Activity logs
- Dependencies
- Filtering & sorting

Excludes:
- Gantt charts
- Billing/time tracking
- Advanced automation
- Notifications

---

## 3. Task Enhancements

### 3.1 Assignment
- Tasks support assignee_user_id
- Must belong to same company
- Optional field

### 3.2 Priority
Values:
- Low
- Medium
- High
- Critical

Default: Medium

### 3.3 Due Dates
- due_date (required: no)
- due_time (optional)

### 3.4 Status

```text
Pending
In Progress
Blocked
In Review
Done
Cancelled
```

---

## 4. Labels

- Reusable labels
- Many-to-many with tasks

Fields:
- id
- name
- color
- company_id

---

## 5. Comments

- Task-level comments
- user_id + content
- editable by owner

---

## 6. Activity Logs

Tracks:
- status changes
- assignment changes
- priority changes
- due date updates

---

## 7. Dependencies

Types:
- Blocks
- Blocked By

Rules:
- Cannot complete blocked task
- Prevent circular dependencies

---

## 8. Filtering

Filter by:
- assignee
- status
- priority
- due date
- labels

Sort by:
- due date
- priority
- created date

---

## 9. Project Enhancements

- Project members
- Dashboard:
  - tasks by status
  - overdue tasks
  - user workload

---

## 10. Data Model

### Task
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

---

## 11. Security

- Company isolation enforced
- Only authorized users modify tasks

---

## 12. Acceptance Criteria

- Tasks assignable
- Dependencies enforced
- Comments functional
- Filtering works
- Dashboard shows insights

---

## 13. Future

- Notifications
- Automation
- Time tracking
- Sprint planning
