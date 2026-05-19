use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use tracing::{info, warn};

use crate::ai::AiError;
use crate::app_state::AppState;
use crate::auth_route;
use crate::types::{
    ApiErrorBody, ListProviderModelsRequest, ListProviderModelsResponse, ProviderCatalogResponse,
    ProviderId,
};

pub async fn list_supported_providers(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<ProviderCatalogResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    info!(has_cookie, "api: GET /api/v1/ai/providers");

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                "api: GET /api/v1/ai/providers -> auth error response"
            );
            (status, json)
        })?;

    let providers = state.ai_providers.supported();
    info!(
        user_id = %user.id,
        provider_count = providers.len(),
        "api: GET /api/v1/ai/providers -> 200 OK"
    );

    Ok(Json(ProviderCatalogResponse { providers }))
}

pub async fn list_provider_models(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(provider_id): Path<String>,
    Json(payload): Json<ListProviderModelsRequest>,
) -> Result<Json<ListProviderModelsResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    info!(
        has_cookie,
        provider_id = %provider_id,
        "api: POST /api/v1/ai/providers/:provider_id/models"
    );

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                provider_id = %provider_id,
                "api: POST /api/v1/ai/providers/:provider_id/models -> auth error response"
            );
            (status, json)
        })?;

    let provider = provider_id
        .parse::<ProviderId>()
        .map_err(|_| not_found("AI provider not found."))?;

    let api_key = payload.api_key.trim();
    if api_key.is_empty() {
        return Err(bad_request("API key is required."));
    }

    let models = state
        .ai_providers
        .get(provider)
        .list_models(api_key)
        .await
        .map_err(|e| {
            warn!(
                error = %e,
                user_id = %user.id,
                provider = provider.as_str(),
                "list_provider_models: provider model discovery failed"
            );
            map_ai_error(e)
        })?;

    info!(
        user_id = %user.id,
        provider = provider.as_str(),
        model_count = models.len(),
        "api: POST /api/v1/ai/providers/:provider_id/models -> 200 OK"
    );

    Ok(Json(ListProviderModelsResponse { provider, models }))
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

fn not_found(message: impl Into<String>) -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiErrorBody {
            message: message.into(),
            ..Default::default()
        }),
    )
}
