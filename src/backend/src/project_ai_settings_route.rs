use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use tracing::{info, warn};
use uuid::Uuid;

use crate::ai::AiError;
use crate::app_state::AppState;
use crate::auth_route;
use crate::project_ai_settings_service::ProjectAiSettingsService;
use crate::tenant_scope_service::TenantScopeService;
use crate::types::{
    ApiErrorBody, ListProviderModelsResponse, ProjectAiModelsRequest, ProjectAiSettingsResponse,
    UpdateProjectAiSettingsRequest,
};

pub async fn get_project_ai_settings(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectAiSettingsResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    info!(
        has_cookie,
        %project_id,
        "api: GET /api/v1/projects/:id/ai-settings"
    );

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                "api: GET /api/v1/projects/:id/ai-settings -> auth error response"
            );
            (status, json)
        })?;
    let scope = TenantScopeService::from_session(&user)?;
    ProjectAiSettingsService::authorize_manage(&state, project_id, scope).await?;

    let response = ProjectAiSettingsService::load_response(&state, project_id)
        .await
        .map_err(|e| {
            warn!(
                error = %e,
                user_id = %user.id,
                %project_id,
                "get_project_ai_settings: load response failed"
            );
            internal_error()
        })?;

    info!(
        user_id = %user.id,
        %project_id,
        has_api_key = response.has_api_key,
        "api: GET /api/v1/projects/:id/ai-settings -> 200 OK"
    );

    Ok(Json(response))
}

pub async fn update_project_ai_settings(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<UpdateProjectAiSettingsRequest>,
) -> Result<Json<ProjectAiSettingsResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    info!(
        has_cookie,
        %project_id,
        provider = payload.provider.as_str(),
        model_count = payload.allowed_models.len(),
        has_api_key = payload.api_key.as_ref().is_some_and(|s| !s.trim().is_empty()),
        clear_api_key = payload.clear_api_key,
        "api: PUT /api/v1/projects/:id/ai-settings (body summarized; API key not logged)"
    );

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                "api: PUT /api/v1/projects/:id/ai-settings -> auth error response"
            );
            (status, json)
        })?;
    let scope = TenantScopeService::from_session(&user)?;
    ProjectAiSettingsService::authorize_manage(&state, project_id, scope).await?;

    if payload.clear_api_key {
        if payload
            .api_key
            .as_ref()
            .is_some_and(|s| !s.trim().is_empty())
        {
            return Err(bad_request(
                "Clear API key or provide a replacement, not both.",
            ));
        }
        ProjectAiSettingsService::upsert(
            &state,
            project_id,
            user.id,
            payload.provider,
            Vec::new(),
            payload.monthly_token_limit,
            payload.usage_tracking_enabled,
            None,
            true,
        )
        .await?;

        let response = ProjectAiSettingsService::load_response(&state, project_id)
            .await
            .map_err(|e| {
                warn!(
                    error = %e,
                    user_id = %user.id,
                    %project_id,
                    "update_project_ai_settings: load response after clear failed"
                );
                internal_error()
            })?;
        return Ok(Json(response));
    }

    let allowed_models = normalize_models(payload.allowed_models);
    if allowed_models.is_empty() {
        return Err(bad_request("Select at least one AI model."));
    }
    if let Some(limit) = payload.monthly_token_limit {
        if limit <= 0 {
            return Err(bad_request(
                "Monthly token limit must be greater than zero.",
            ));
        }
    }

    let api_key = payload
        .api_key
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let existing_api_key = if api_key.is_none() {
        ProjectAiSettingsService::decrypt_existing_api_key(&state, project_id)
            .await
            .map_err(|e| {
                warn!(
                    error = %e,
                    user_id = %user.id,
                    %project_id,
                    "update_project_ai_settings: decrypt existing key failed"
                );
                internal_error()
            })?
    } else {
        None
    };
    let validation_key = api_key
        .as_deref()
        .or(existing_api_key.as_deref())
        .ok_or_else(|| bad_request("API key is required."))?;

    let known_models = state
        .ai_providers
        .get(payload.provider)
        .list_models(validation_key)
        .await
        .map_err(|e| {
            warn!(
                error = %e,
                user_id = %user.id,
                %project_id,
                provider = payload.provider.as_str(),
                "update_project_ai_settings: model validation failed"
            );
            map_ai_error(e)
        })?;

    for model in &allowed_models {
        if !known_models.contains(model) {
            return Err(bad_request(
                "Selected model is not supported by this provider.",
            ));
        }
    }

    ProjectAiSettingsService::upsert(
        &state,
        project_id,
        user.id,
        payload.provider,
        allowed_models,
        payload.monthly_token_limit,
        payload.usage_tracking_enabled,
        api_key,
        false,
    )
    .await?;

    let response = ProjectAiSettingsService::load_response(&state, project_id)
        .await
        .map_err(|e| {
            warn!(
                error = %e,
                user_id = %user.id,
                %project_id,
                "update_project_ai_settings: load response after save failed"
            );
            internal_error()
        })?;

    info!(
        user_id = %user.id,
        %project_id,
        provider = response.provider.map(|p| p.as_str()).unwrap_or("none"),
        model_count = response.allowed_models.len(),
        "api: PUT /api/v1/projects/:id/ai-settings -> 200 OK"
    );

    Ok(Json(response))
}

pub async fn list_project_ai_provider_models(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<ProjectAiModelsRequest>,
) -> Result<Json<ListProviderModelsResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    info!(
        has_cookie,
        %project_id,
        provider = payload.provider.as_str(),
        "api: POST /api/v1/projects/:id/ai-settings/provider-models (body summarized; API key not logged)"
    );

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                "api: POST /api/v1/projects/:id/ai-settings/provider-models -> auth error response"
            );
            (status, json)
        })?;
    let scope = TenantScopeService::from_session(&user)?;
    ProjectAiSettingsService::authorize_manage(&state, project_id, scope).await?;

    let submitted_api_key = payload.api_key.trim();
    let stored_api_key = if submitted_api_key.is_empty() {
        ProjectAiSettingsService::decrypt_existing_api_key(&state, project_id)
            .await
            .map_err(|e| {
                warn!(
                    error = %e,
                    user_id = %user.id,
                    %project_id,
                    "list_project_ai_provider_models: decrypt existing key failed"
                );
                internal_error()
            })?
    } else {
        None
    };
    let api_key = if submitted_api_key.is_empty() {
        stored_api_key
            .as_deref()
            .ok_or_else(|| bad_request("API key is required."))?
    } else {
        submitted_api_key
    };

    let models = state
        .ai_providers
        .get(payload.provider)
        .list_models(api_key)
        .await
        .map_err(|e| {
            warn!(
                error = %e,
                user_id = %user.id,
                %project_id,
                provider = payload.provider.as_str(),
                "list_project_ai_provider_models: provider model discovery failed"
            );
            map_ai_error(e)
        })?;

    info!(
        user_id = %user.id,
        %project_id,
        provider = payload.provider.as_str(),
        model_count = models.len(),
        "api: POST /api/v1/projects/:id/ai-settings/provider-models -> 200 OK"
    );

    Ok(Json(ListProviderModelsResponse {
        provider: payload.provider,
        models,
    }))
}

fn normalize_models(models: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for model in models {
        let trimmed = model.trim();
        if trimmed.is_empty() {
            continue;
        }
        if normalized
            .iter()
            .any(|existing: &String| existing == trimmed)
        {
            continue;
        }
        normalized.push(trimmed.to_string());
    }
    normalized
}

fn map_ai_error(error: AiError) -> (StatusCode, Json<ApiErrorBody>) {
    let status = match error {
        AiError::Network | AiError::Provider(_) | AiError::Decode | AiError::Empty => {
            StatusCode::BAD_GATEWAY
        }
    };

    (
        status,
        Json(ApiErrorBody {
            message: "Could not list provider models right now. Please try again.".into(),
            ..Default::default()
        }),
    )
}

fn bad_request(message: impl Into<String>) -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiErrorBody {
            message: message.into(),
            ..Default::default()
        }),
    )
}

fn internal_error() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiErrorBody {
            message: "Something went wrong. Please try again.".into(),
            ..Default::default()
        }),
    )
}
