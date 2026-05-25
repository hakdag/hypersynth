use axum::extract::Query;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;

use crate::ai_usage_query_helpers::bad_request;
use crate::audit_log_query_service::AuditLogQueryService;
use crate::auth_route::require_system_admin;
use crate::tx_extractor::missing_tx_error;
use crate::types::{AdminAuditLogsListQuery, AdminAuditLogsListResponse, ApiErrorBody, Tx};

pub async fn list_admin_audit_logs(
    tx: Tx,
    jar: CookieJar,
    Query(query): Query<AdminAuditLogsListQuery>,
) -> Result<Json<AdminAuditLogsListResponse>, (StatusCode, Json<ApiErrorBody>)> {
    if let (Some(from), Some(to)) = (query.from, query.to) {
        if from >= to {
            return Err(bad_request("'from' must be before 'to'."));
        }
    }

    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;
    let _admin_email = require_system_admin(conn, &jar).await?;

    let response = AuditLogQueryService::list(conn, &query)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorBody::msg(
                    "Something went wrong. Please try again.",
                )),
            )
        })?;

    Ok(Json(response))
}
