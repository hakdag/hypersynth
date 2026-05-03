# Phase 0 Sub-Feature Index

## Source

Derived from the Phase 0 PRD for the AI-Driven Project Management System.

## Delivery Strategy

The project starts from scratch. The sub-features below are ordered so that each item can be designed, implemented, tested, and deployed as independently as possible. Some dependencies are unavoidable, but each sub-feature is scoped to provide a self-contained increment.

## Sub-Features

- [SF-00 — Project Initialization and Application Shell](./01-sf-00-project-initialization-and-application-shell.md)
- [SF-01 — User Registration](./02-sf-01-user-registration.md)
- [SF-02 — User Login and Logout](./03-sf-02-user-login-and-logout.md)
- [SF-03 — Personal Workspace and Data Isolation](./04-sf-03-personal-workspace-and-data-isolation.md)
- [SF-04 — Project Creation](./05-sf-04-project-creation.md)
- [SF-05 — Project Listing and Detail View](./06-sf-05-project-listing-and-detail-view.md)
- [SF-06 — Project Editing and Status Management](./07-sf-06-project-editing-and-status-management.md)
- [SF-07 — Feature Creation Under Project](./08-sf-07-feature-creation-under-project.md)
- [SF-08 — Feature Listing and Detail View](./09-sf-08-feature-listing-and-detail-view.md)
- [SF-09 — Feature Editing and Status Management](./10-sf-09-feature-editing-and-status-management.md)
- [SF-10 — Manual Task Creation Under Feature](./11-sf-10-manual-task-creation-under-feature.md)
- [SF-11 — Task Listing and Detail View](./12-sf-11-task-listing-and-detail-view.md)
- [SF-12 — Task Editing and Status Management](./13-sf-12-task-editing-and-status-management.md)
- [SF-13 — Project Document Upload](./14-sf-13-project-document-upload.md)
- [SF-14 — Project Document Listing and Selection](./15-sf-14-project-document-listing-and-selection.md)
- [SF-15 — AI API Key Configuration Per Project](./16-sf-15-ai-api-key-configuration-per-project.md)
- [SF-16 — AI Enhancement of Project Requirements](./17-sf-16-ai-enhancement-of-project-requirements.md)
- [SF-17 — AI Enhancement of Feature Requirements](./18-sf-17-ai-enhancement-of-feature-requirements.md)
- [SF-18 — AI Task Generation From Feature Requirements](./19-sf-18-ai-task-generation-from-feature-requirements.md)

## Recommended Delivery Order

1. Build the project shell and authentication foundation.
2. Enforce personal workspace isolation before adding user-owned business data.
3. Deliver Project → Feature → Task management manually.
4. Add document upload and document selection.
5. Add AI configuration and AI-assisted workflows one capability at a time.

## Notes

- No technology stack is prescribed in these files.
- Each sub-feature avoids coupling itself to unrelated future enhancements.
- AI-generated outputs are reviewed by the user before modifying persisted project, feature, or task data.
