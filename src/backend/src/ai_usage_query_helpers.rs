use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Duration, Utc};

use crate::types::ApiErrorBody;

pub const DEFAULT_RANGE_DAYS: i64 = 30;
pub const DEFAULT_LIST_LIMIT: i64 = 50;
pub const MAX_LIST_LIMIT: i64 = 200;

pub fn resolve_range(
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
) -> Result<(DateTime<Utc>, DateTime<Utc>), (StatusCode, Json<ApiErrorBody>)> {
    let to = to.unwrap_or_else(Utc::now);
    let from = from.unwrap_or_else(|| to - Duration::days(DEFAULT_RANGE_DAYS));

    if from >= to {
        return Err(bad_request("'from' must be before 'to'."));
    }

    Ok((from, to))
}

pub fn pagination_limit(limit: Option<i64>) -> i64 {
    limit
        .unwrap_or(DEFAULT_LIST_LIMIT)
        .clamp(1, MAX_LIST_LIMIT)
}

pub fn pagination_offset(offset: Option<i64>) -> i64 {
    offset.unwrap_or(0).max(0)
}

pub fn bad_request(message: impl Into<String>) -> (StatusCode, Json<ApiErrorBody>) {
    (StatusCode::BAD_REQUEST, Json(ApiErrorBody::msg(message)))
}

pub fn internal_error() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiErrorBody::msg("Something went wrong. Please try again.")),
    )
}
