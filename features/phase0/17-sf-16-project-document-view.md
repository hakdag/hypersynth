# SF-16 — Project Document View

## Purpose

Allow a user to view supported uploaded documents in the UI.

## Summary

This sub-feature adds an in-app document viewing experience for supported file types. Text files and images can be opened in a modal, while unsupported files remain available through download.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Provide a view action for supported documents from the document list.
- Open supported documents in a modal.
- Support plain text files in a readable text view.
- Support image files in an image preview.
- Clearly indicate when preview is not supported for a document.
- Allow the modal to be closed without leaving the project detail view.
- Ensure viewed documents belong to the selected project and authenticated user.

## Out of Scope

- Document upload.
- Document deletion.
- Editing document content.
- Preview support for PDFs, office documents, archives, audio, or video.
- Document selection for AI requests.
- Document content extraction or indexing for AI.

## Dependencies

- SF-14 Project Document Listing
- SF-15 Project Document Download

## Independent Deployment Notes

Can be deployed after secure document retrieval exists. Unsupported files should continue to be handled by download rather than blocking document management.

## User Stories

- As a user, I want to preview text documents in the project UI so that I can inspect project context quickly.
- As a user, I want to preview image documents in the project UI so that I do not have to download each image first.
- As a user, I want unsupported files to be clearly marked so that I understand why they cannot be viewed inline.

## Acceptance Criteria

- Supported text documents can be opened in a modal.
- Supported image documents can be opened in a modal.
- Unsupported document types do not show a misleading preview.
- The modal can be closed and returns the user to the same project detail context.
- A user cannot view documents from another user’s project.
- Preview loading, unsupported type, unauthorized access, and storage failures show clear states.

## Data Requirements

- Reads Document records by id and project_id.
- Uses stored file path, content type, original filename, and metadata to determine preview support.

## Security and Isolation Requirements

- Document viewing must be validated through project ownership.
- Preview URLs or content endpoints must not expose documents across users or projects.
- Unsupported content must not be rendered as trusted HTML.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.

