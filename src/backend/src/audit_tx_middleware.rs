use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::extract::{ConnectInfo, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_extra::extract::CookieJar;
use serde_json::Value;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth_route::{client_ip, resolve_current_principal, user_agent};
use crate::tx_extractor::CommitAuditOnFailure;
use crate::types::{
    ApiErrorBody, AuditActor, AuditContext, SessionPrincipal, SessionUser, SharedAuditContext, Tx,
};

/// Per-request transaction + audit-context middleware.
///
/// For every request that flows through this middleware:
/// 1. Acquire a connection from the pool and BEGIN a transaction.
/// 2. Resolve the calling principal (anonymous, regular user, or System
///    Admin) using the cookie. The resolver may revoke a session if the
///    user or company has been disabled.
/// 3. Set transaction-local GUCs (`app.actor`, `app.request_id`,
///    `app.ip_address`, `app.user_agent`) so the row-change trigger can
///    stamp them onto every audit row produced by this request.
/// 4. Install the `Tx` handle and an `AuditContext` snapshot into the
///    request extensions so handlers can extract them.
/// 5. Run the inner handler.
/// 6. Commit or roll back based on the response status (with an opt-in
///    `CommitAuditOnFailure` marker for handlers like login-failure
///    that need audit-related writes to persist on error responses).
pub async fn audit_tx_middleware(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Response {
    let request_id = Uuid::new_v4();
    let ip_string = client_ip(&peer, request.headers());
    let ip_addr = ip_string.parse::<IpAddr>().ok();
    let ua_string = user_agent(request.headers());
    let ua_opt = if ua_string.is_empty() {
        None
    } else {
        Some(ua_string)
    };

    let mut tx = match state.pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = %e, "audit_tx_middleware: pool.begin failed");
            return pool_unavailable_response();
        }
    };

    let principal_result = resolve_current_principal(&mut *tx, &jar).await;

    let actor_value = match principal_result {
        Ok(Some(principal)) => {
            let actor = actor_from_principal(principal);
            serde_json::to_value(&actor).unwrap_or(Value::Null)
        }
        Ok(None) => serde_json::to_value(AuditActor::anonymous()).unwrap_or(Value::Null),
        Err(disabled_err) => {
            if let Err(e) = tx.commit().await {
                tracing::error!(
                    error = %e,
                    "audit_tx_middleware: commit failed after disabled-session revoke"
                );
            }
            return disabled_err.into_response();
        }
    };

    if let Err(e) = set_request_gucs(
        &mut *tx,
        &actor_value,
        request_id,
        ip_addr.as_ref(),
        ua_opt.as_deref(),
    )
    .await
    {
        tracing::error!(error = %e, "audit_tx_middleware: set_config failed");
        let _ = tx.rollback().await;
        return pool_unavailable_response();
    }

    let audit_ctx: SharedAuditContext = Arc::new(AuditContext {
        actor: actor_value,
        request_id,
        ip_address: ip_addr,
        user_agent: ua_opt,
    });

    let tx_handle = Tx::new(tx);
    let post_handle = Arc::clone(&tx_handle.0);

    request.extensions_mut().insert(tx_handle);
    request.extensions_mut().insert(audit_ctx);

    let response = next.run(request).await;

    let should_commit = response.status().is_success()
        || response.status().is_redirection()
        || response.extensions().get::<CommitAuditOnFailure>().is_some();

    let mut guard = post_handle.lock().await;
    if let Some(tx) = guard.take() {
        if should_commit {
            if let Err(e) = tx.commit().await {
                tracing::error!(error = %e, "audit_tx_middleware: commit failed");
                return commit_failed_response();
            }
        } else if let Err(e) = tx.rollback().await {
            tracing::error!(error = %e, "audit_tx_middleware: rollback failed");
        }
    }

    response
}

async fn set_request_gucs(
    conn: &mut sqlx::PgConnection,
    actor: &Value,
    request_id: Uuid,
    ip: Option<&IpAddr>,
    ua: Option<&str>,
) -> Result<(), sqlx::Error> {
    let actor_str = serde_json::to_string(actor).unwrap_or_else(|_| "null".to_string());
    let ip_str = ip.map(IpAddr::to_string).unwrap_or_default();
    let ua_str = ua.unwrap_or("");

    sqlx::query(
        r#"
        SELECT
            set_config('app.actor',       $1, true),
            set_config('app.request_id',  $2, true),
            set_config('app.ip_address',  $3, true),
            set_config('app.user_agent',  $4, true)
        "#,
    )
    .bind(actor_str)
    .bind(request_id.to_string())
    .bind(ip_str)
    .bind(ua_str)
    .execute(conn)
    .await?;

    Ok(())
}

fn actor_from_principal(principal: SessionPrincipal) -> AuditActor {
    match principal {
        SessionPrincipal::User(SessionUser {
            id,
            email,
            account_type,
            company_id,
            ..
        }) => AuditActor {
            system_admin: false,
            user_id: Some(id),
            email: Some(email),
            account_type: Some(account_type.as_db_value().to_string()),
            company_id,
        },
        SessionPrincipal::SystemAdmin { email } => AuditActor {
            system_admin: true,
            user_id: None,
            email: Some(email),
            account_type: None,
            company_id: None,
        },
    }
}

fn pool_unavailable_response() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ApiErrorBody {
            message: "The service is temporarily unavailable. Please try again.".into(),
            ..Default::default()
        }),
    )
        .into_response()
}

fn commit_failed_response() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiErrorBody {
            message: "Failed to persist changes. Please try again.".into(),
            ..Default::default()
        }),
    )
        .into_response()
}

