use axum::http::StatusCode;
use axum::Json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::types::{ApiErrorBody, TenantScope};

pub const ALLOWED_TASK_STATUSES: &[&str] = &[
    "Pending",
    "In Progress",
    "Blocked",
    "In Review",
    "Done",
    "Cancelled",
];
pub const TERMINAL_TASK_STATUSES: &[&str] = &["Done", "Cancelled"];

pub fn is_allowed_task_status(status: &str) -> bool {
    ALLOWED_TASK_STATUSES.contains(&status)
}

pub fn validate_task_status_trimmed(status: &str) -> Result<&str, (StatusCode, Json<ApiErrorBody>)> {
    if is_allowed_task_status(status) {
        Ok(status)
    } else {
        Err(bad_request(
            "Status must be Pending, In Progress, Blocked, In Review, Done, or Cancelled.",
        ))
    }
}

/// SF-32 hook point for SF-37 dependency checks.
/// For now, marking a task as Done is always permitted.
pub async fn validate_may_mark_done(
    _pool: &PgPool,
    _scope: TenantScope,
    _task_id: Uuid,
    _project_id: Uuid,
) -> Result<(), (StatusCode, Json<ApiErrorBody>)> {
    Ok(())
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
