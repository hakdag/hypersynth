# SF-21 — AI Provider Settings

## Purpose

Define how the system supports multiple AI providers so that company and personal AI configurations can target a specific provider and a specific set of allowed models from that provider.

## Summary

This sub-feature introduces first-class AI provider support. The system maintains a catalog of supported AI providers (for example, Anthropic, OpenAI), and exposes a way to:

- Choose which provider a stored AI configuration targets.
- Select which models from that provider are allowed for use.
- Route AI operations to the chosen provider at runtime through a provider-agnostic interface.

Provider selection is a building block consumed by Company and Personal AI Settings (SF-22) and by AI usage tracking (SF-23). This sub-feature owns the concept of "which providers and models exist"; it does not own API keys or per-scope settings.

## Scrum-Oriented Delivery Principle

This sub-feature is designed to be independently implementable, testable, and deployable. It ships the provider abstraction before consumers need it so downstream settings features can rely on a stable provider concept.

## Scope

- Define a server-side catalog of supported AI providers, each identified by a stable string identifier.
- For each provider, declare the list of models the system knows how to call.
- Route AI operations through a provider-agnostic interface so adding a new provider is a localized change at the integration layer.
- Validate that a saved provider value is one of the supported providers and that any `allowed_models` selection only contains models known for that provider.
- Expose the supported providers and their model lists to the frontend so settings screens (SF-22) can render the choices.

## Out of Scope

- Storing per-company or per-user provider API keys (covered by SF-22).
- AI usage tracking and reporting (covered by SF-23).
- Per-request provider override by end users at AI call time.
- Provider failover or multi-provider fan-out for a single request.
- Dynamic registration of providers at runtime by non-developers.

## Dependencies

- None at the data layer. May be deployed before SF-22.

## Independent Deployment Notes

Can be deployed independently. Until SF-22 ships, the provider catalog is read-only configuration that the AI call layer consumes; no user-facing screen is required.

## User Stories

- As a platform engineer, I want a single place that declares which AI providers and models are supported so that adding a new provider is a localized change.
- As a Company Admin or personal user (via SF-22), I want to pick a provider and a model list so that AI features run against the provider I am paying for.

## Acceptance Criteria

- The system exposes the list of supported providers and, per provider, the list of supported models.
- A stored AI configuration that references an unsupported provider or an unknown model is rejected with a validation error.
- AI operations invoked through the provider-agnostic interface dispatch to the correct provider implementation based on the configuration's `provider` value.
- Adding a new provider does not require changes to consumer features other than registering the provider in the catalog and providing its integration adapter.

## Data Requirements

- Provider identifier: stable string (for example, `anthropic`, `openai`).
- Per-provider list of supported model identifiers.
- These values are consumed by AI Settings records via the `provider` and `allowed_models` fields defined in SF-22; this sub-feature does not introduce its own persisted table.

## Security and Isolation Requirements

- The provider catalog itself contains no secret material.
- Provider selection must never leak credentials; API keys remain the responsibility of SF-22 storage.
- Adapter implementations must not log request or response bodies that could include credentials.

## Deployment Readiness Checklist

- The sub-feature can be enabled without requiring non-dependent future sub-features.
- Existing completed flows remain functional after deployment; the existing single-provider behavior continues to work.
- Empty, validation, success, and failure states are handled for provider and model validation.
- The feature can be tested with clear pass/fail acceptance criteria.
