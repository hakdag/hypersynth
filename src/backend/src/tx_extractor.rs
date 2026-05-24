use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::Json;

use crate::types::{ApiErrorBody, SharedAuditContext, Tx};

/// Pulls the request-scoped transaction handle out of request extensions.
///
/// Returns 500 if the audit middleware did not install one. In production
/// this should be unreachable; all routes that mutate or read business
/// data must be served behind `audit_tx_middleware`.
impl<S> FromRequestParts<S> for Tx
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ApiErrorBody>);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Tx>()
            .cloned()
            .ok_or_else(missing_tx_error)
    }
}

/// Pulls the per-request `AuditContext` out of extensions for fire-and-forget
/// non-data audit event recording (see `AuditEventsService`).
pub struct AuditCtx(pub SharedAuditContext);

impl<S> FromRequestParts<S> for AuditCtx
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ApiErrorBody>);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<SharedAuditContext>()
            .cloned()
            .map(AuditCtx)
            .ok_or_else(missing_audit_ctx_error)
    }
}

/// Marker placed into response extensions by handlers that need their
/// audit-event work durably committed even when the response is an
/// error (e.g. System Admin login failure). The middleware consults
/// this marker before deciding to roll back.
#[derive(Debug, Clone, Copy)]
pub struct CommitAuditOnFailure;

pub fn missing_tx_error() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiErrorBody {
            message: "Request transaction is not available.".into(),
            ..Default::default()
        }),
    )
}

fn missing_audit_ctx_error() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiErrorBody {
            message: "Request audit context is not available.".into(),
            ..Default::default()
        }),
    )
}
