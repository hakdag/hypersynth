use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::auth_route::require_system_admin;
use crate::tx_extractor::missing_tx_error;
use crate::types::{
    AdminUserDetail, AdminUserSummary, AdminUsersListQuery, ApiErrorBody, Tx,
    UpdateUserStatusRequest, UserStatus,
};

const DEFAULT_LIST_LIMIT: i64 = 50;
const MAX_LIST_LIMIT: i64 = 200;

pub async fn list_admin_users(
    tx: Tx,
    jar: CookieJar,
    Query(query): Query<AdminUsersListQuery>,
) -> Result<Json<Vec<AdminUserSummary>>, (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;
    let _admin_email = require_system_admin(conn, &jar).await?;

    let limit = query
        .limit
        .unwrap_or(DEFAULT_LIST_LIMIT)
        .clamp(1, MAX_LIST_LIMIT);
    let offset = query.offset.unwrap_or(0).max(0);

    let search = query
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let account_type = query
        .account_type
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let status = query
        .status
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let rows = sqlx::query_as::<_, AdminUserSummary>(
        r#"
        SELECT
            u.id,
            u.fullname AS full_name,
            u.email,
            u.username,
            u.account_type,
            u.role,
            u.status,
            cu.company_id,
            c.name AS company_name,
            u.created_at
        FROM users u
        LEFT JOIN company_users cu ON cu.user_id = u.id
        LEFT JOIN companies c ON c.id = cu.company_id
        WHERE (
            $1::text IS NULL
            OR u.email ILIKE '%' || $1 || '%'
            OR u.fullname ILIKE '%' || $1 || '%'
            OR u.username ILIKE '%' || $1 || '%'
            OR c.name ILIKE '%' || $1 || '%'
        )
        AND ($2::text IS NULL OR u.account_type = $2)
        AND ($3::text IS NULL OR u.status = $3)
        AND ($4::uuid IS NULL OR cu.company_id = $4)
        ORDER BY lower(u.email) ASC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(search.as_deref())
    .bind(account_type.as_deref())
    .bind(status.as_deref())
    .bind(query.company_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&mut **conn)
    .await
    .map_err(|_| internal_error())?;

    Ok(Json(rows))
}

pub async fn get_admin_user(
    tx: Tx,
    jar: CookieJar,
    Path(user_id): Path<Uuid>,
) -> Result<Json<AdminUserDetail>, (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;
    let _admin_email = require_system_admin(conn, &jar).await?;

    let detail = fetch_admin_user_detail(conn, user_id)
        .await?
        .ok_or_else(not_found)?;

    Ok(Json(detail))
}

pub async fn set_admin_user_status(
    tx: Tx,
    jar: CookieJar,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateUserStatusRequest>,
) -> Result<Json<AdminUserDetail>, (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;
    let _admin_email = require_system_admin(conn, &jar).await?;

    match payload.status {
        UserStatus::Active | UserStatus::Disabled => {}
        UserStatus::PendingInvitation => {
            return Err(bad_request(
                "Only active or disabled status can be set by a system administrator.",
            ));
        }
    }

    let previous_status: Option<String> =
        sqlx::query_scalar("SELECT status FROM users WHERE id = $1 FOR UPDATE")
            .bind(user_id)
            .fetch_optional(&mut **conn)
            .await
            .map_err(|_| internal_error())?;

    if previous_status.is_none() {
        return Err(not_found());
    }

    sqlx::query(
        r#"
        UPDATE users
        SET status = $2, updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .bind(payload.status.as_db_value())
    .execute(&mut **conn)
    .await
    .map_err(|_| internal_error())?;

    if payload.status == UserStatus::Disabled {
        sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut **conn)
            .await
            .map_err(|_| internal_error())?;
    }

    let detail = fetch_admin_user_detail(conn, user_id)
        .await?
        .ok_or_else(not_found)?;

    Ok(Json(detail))
}

pub async fn reset_admin_user_access(
    tx: Tx,
    jar: CookieJar,
    Path(user_id): Path<Uuid>,
) -> Result<Json<AdminUserDetail>, (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;
    let _admin_email = require_system_admin(conn, &jar).await?;

    let exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&mut **conn)
        .await
        .map_err(|_| internal_error())?;

    if exists.is_none() {
        return Err(not_found());
    }

    sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut **conn)
        .await
        .map_err(|_| internal_error())?;

    let detail = fetch_admin_user_detail(conn, user_id)
        .await?
        .ok_or_else(not_found)?;

    Ok(Json(detail))
}

async fn fetch_admin_user_detail(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<Option<AdminUserDetail>, (StatusCode, Json<ApiErrorBody>)> {
    let detail = sqlx::query_as::<_, AdminUserDetail>(
        r#"
        SELECT
            u.id,
            u.fullname AS full_name,
            u.display_name,
            u.email,
            u.username,
            u.account_type,
            u.role,
            u.status,
            u.timezone,
            cu.company_id,
            c.name AS company_name,
            u.created_at,
            u.updated_at
        FROM users u
        LEFT JOIN company_users cu ON cu.user_id = u.id
        LEFT JOIN companies c ON c.id = cu.company_id
        WHERE u.id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|_| internal_error())?;

    let Some(mut detail) = detail else {
        return Ok(None);
    };

    detail.active_session_count = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM sessions
        WHERE user_id = $1 AND expires_at > now()
        "#,
    )
    .bind(user_id)
    .fetch_one(&mut *conn)
    .await
    .map_err(|_| internal_error())?;

    Ok(Some(detail))
}

fn not_found() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiErrorBody::msg("User not found.")),
    )
}

fn bad_request(message: impl Into<String>) -> (StatusCode, Json<ApiErrorBody>) {
    (StatusCode::BAD_REQUEST, Json(ApiErrorBody::msg(message)))
}

fn internal_error() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiErrorBody::msg("Something went wrong. Please try again.")),
    )
}
