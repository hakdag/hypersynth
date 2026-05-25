use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use uuid::Uuid;

use crate::ai_usage_query_helpers::bad_request;
use crate::auth_route::require_system_admin;
use crate::tx_extractor::missing_tx_error;
use crate::types::{
    AdminInvitationSummary, AdminInvitationsListQuery, AdminInvitationsListResponse,
    ApiErrorBody, InvitationStatus, Tx,
};

const DEFAULT_LIST_LIMIT: i64 = 50;
const MAX_LIST_LIMIT: i64 = 200;

pub async fn list_admin_invitations(
    tx: Tx,
    jar: CookieJar,
    Query(query): Query<AdminInvitationsListQuery>,
) -> Result<Json<AdminInvitationsListResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let created_from = query.from;
    let created_to = query.to;
    if let (Some(from), Some(to)) = (created_from, created_to) {
        if from > to {
            return Err(bad_request("'from' day must not be after 'to' day."));
        }
    }

    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;
    let _admin_email = require_system_admin(conn, &jar).await?;

    let _ = sqlx::query(
        r#"
        UPDATE invitations
        SET status = $1
        WHERE status = $2
          AND expires_at < now()
        "#,
    )
    .bind(InvitationStatus::Expired.as_db_value())
    .bind(InvitationStatus::Pending.as_db_value())
    .execute(&mut **conn)
    .await
    .map_err(|_| internal_error())?;

    let limit = query
        .limit
        .unwrap_or(DEFAULT_LIST_LIMIT)
        .clamp(1, MAX_LIST_LIMIT);
    let offset = query.offset.unwrap_or(0).max(0);

    let status = query
        .status
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)::bigint
        FROM invitations i
        JOIN companies c ON c.id = i.company_id
        JOIN users u ON u.id = i.invited_by_user_id
        WHERE ($1::uuid IS NULL OR i.company_id = $1)
          AND (
              ($2::text IS NOT NULL AND i.status = $2)
              OR ($2::text IS NULL AND i.status IN ('pending', 'expired'))
          )
          AND ($3::timestamptz IS NULL OR i.created_at >= $3)
          AND ($4::timestamptz IS NULL OR i.created_at <= $4)
        "#,
    )
    .bind(query.company_id)
    .bind(status.as_deref())
    .bind(created_from)
    .bind(created_to)
    .fetch_one(&mut **conn)
    .await
    .map_err(|_| internal_error())?;

    let items = sqlx::query_as::<_, AdminInvitationSummary>(
        r#"
        SELECT
            i.id,
            i.company_id,
            c.name AS company_name,
            i.invited_by_user_id,
            u.fullname AS inviter_name,
            u.email AS inviter_email,
            i.invited_email,
            i.invited_role,
            i.status,
            i.expires_at,
            i.created_at
        FROM invitations i
        JOIN companies c ON c.id = i.company_id
        JOIN users u ON u.id = i.invited_by_user_id
        WHERE ($1::uuid IS NULL OR i.company_id = $1)
          AND (
              ($2::text IS NOT NULL AND i.status = $2)
              OR ($2::text IS NULL AND i.status IN ('pending', 'expired'))
          )
          AND ($3::timestamptz IS NULL OR i.created_at >= $3)
          AND ($4::timestamptz IS NULL OR i.created_at <= $4)
        ORDER BY i.created_at DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(query.company_id)
    .bind(status.as_deref())
    .bind(created_from)
    .bind(created_to)
    .bind(limit)
    .bind(offset)
    .fetch_all(&mut **conn)
    .await
    .map_err(|_| internal_error())?;

    Ok(Json(AdminInvitationsListResponse {
        items,
        total: total.0,
        limit,
        offset,
    }))
}

pub async fn cancel_admin_invitation(
    tx: Tx,
    jar: CookieJar,
    Path(invitation_id): Path<Uuid>,
) -> Result<Json<AdminInvitationSummary>, (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;
    let _admin_email = require_system_admin(conn, &jar).await?;

    let updated = sqlx::query_as::<_, AdminInvitationSummary>(
        r#"
        UPDATE invitations i
        SET status = $2
        FROM companies c, users u
        WHERE i.id = $1
          AND i.status = $3
          AND c.id = i.company_id
          AND u.id = i.invited_by_user_id
        RETURNING
            i.id,
            i.company_id,
            c.name AS company_name,
            i.invited_by_user_id,
            u.fullname AS inviter_name,
            u.email AS inviter_email,
            i.invited_email,
            i.invited_role,
            i.status,
            i.expires_at,
            i.created_at
        "#,
    )
    .bind(invitation_id)
    .bind(InvitationStatus::Cancelled.as_db_value())
    .bind(InvitationStatus::Pending.as_db_value())
    .fetch_optional(&mut **conn)
    .await
    .map_err(|_| internal_error())?
    .ok_or_else(not_found)?;

    Ok(Json(updated))
}

fn not_found() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiErrorBody {
            message: "Invitation not found.".into(),
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
