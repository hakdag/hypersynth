# SF-18 — AI Task Generation From Feature Requirements

## Purpose

Allow a user to generate tasks automatically from feature requirements.

## Summary

This sub-feature completes the initial AI-assisted breakdown flow by generating tasks for a feature. Generated tasks remain editable through the regular task lifecycle.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It may depend on earlier foundation capabilities, but it should not require unrelated future features to be completed before it can provide value.

## Scope

- Provide a generate tasks action from a feature detail view.
- Use feature requirements as primary input.
- Use parent project requirements as optional context when available.
- Allow selected project documents to be included as optional context.
- Require the parent project to have an AI API key configured before executing the request.
- Generate a list of task candidates with title and description.
- Allow the user to review generated task candidates before saving.
- Create accepted generated tasks under the selected feature.
- Set generated task status to Pending by default.
- Set created_by to AI or equivalent AI-origin marker.

## Out of Scope

- Automatic task assignment.
- Task estimation.
- Sprint planning.
- AI regeneration or merge conflict handling.
- Approval roles beyond user review.

## Dependencies

- SF-12 Task Editing and Status Management
- SF-14 Project Document Listing and Selection
- SF-15 AI API Key Configuration Per Project

## Independent Deployment Notes

Can be deployed after basic task management and AI configuration exist. It creates tasks but does not require advanced project planning features.

## User Stories

- As a user, I want AI to generate tasks from feature requirements so that I can quickly create an actionable breakdown.
- As a user, I want to review generated tasks before saving so that irrelevant tasks are not added automatically.

## Acceptance Criteria

- AI task generation is available only for features under projects owned by the authenticated user.
- The action requires the parent project to have an AI API key configured.
- Generated task candidates include at least title and description.
- The user can review generated candidates before saving.
- Accepted tasks are created under the selected feature.
- Generated tasks have status Pending by default.
- Generated tasks have created_by set to AI or equivalent.
- If AI request fails, no tasks are created.

## Data Requirements

- Reads Feature.requirements.
- Reads parent Project.requirements and Project.ai_api_key.
- May read selected Document metadata/content as context.
- Creates Task records: feature_id, title, description, status, created_by.

## Security and Isolation Requirements

- Only owner-controlled project, feature, and selected document data are sent to AI.
- AI API key must not be exposed.
- Failed AI requests must not create partial or corrupted task records.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment.
- Empty, validation, success, and failure states are handled.
- User-owned data is protected according to workspace isolation rules.
- The feature can be tested with clear pass/fail acceptance criteria.

