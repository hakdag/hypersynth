use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use serde_json::json;

use crate::app_state::AppState;
use crate::audit_events_service::AuditEventsService;
use crate::auth_route::require_system_admin;
use crate::platform_config_service::PlatformConfigService;
use crate::tx_extractor::{missing_tx_error, AuditCtx};
use crate::types::{
    ApiErrorBody, AuditEventType, PlatformConfig, Tx, UpdatePlatformConfigRequest,
};

pub async fn get_admin_platform_config(
    tx: Tx,
    jar: CookieJar,
) -> Result<Json<PlatformConfig>, (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;
    let _admin_email = require_system_admin(conn, &jar).await?;

    let config = PlatformConfigService::load(conn)
        .await
        .map_err(|_| internal_error())?;

    Ok(Json(config))
}

pub async fn patch_admin_platform_config(
    State(state): State<AppState>,
    tx: Tx,
    jar: CookieJar,
    auditctx: AuditCtx,
    Json(payload): Json<UpdatePlatformConfigRequest>,
) -> Result<Json<PlatformConfig>, (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;
    let admin_email = require_system_admin(conn, &jar).await?;

    let config = PlatformConfigService::update(conn, payload).await?;

    AuditEventsService::record_with_pool(
        &state.pool,
        AuditEventType::GlobalConfigurationChanged,
        &auditctx.0,
        json!({
            "system_admin_email": admin_email,
            "allowed_ai_providers": config.allowed_ai_providers,
            "default_monthly_token_limit": config.default_monthly_token_limit,
            "has_platform_announcement": config.platform_announcement.is_some(),
            "feature_flag_keys": config.feature_flags.keys().collect::<Vec<_>>(),
        }),
    )
    .await;

    Ok(Json(config))
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
