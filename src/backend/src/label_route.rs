use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth_route::require_authenticated_user;
use crate::authorization;
use crate::label_service;
use crate::tenant_scope_service::TenantScopeService;
use crate::tx_extractor::missing_tx_error;
use crate::types::{ApiErrorBody, CreateLabelRequest, Label, LabelResponse, TenantScope, Tx, UpdateLabelRequest};

pub async fn list_labels(
    tx: Tx,
    jar: CookieJar,
) -> Result<Json<Vec<LabelResponse>>, (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;
    let user = require_authenticated_user(conn, &jar).await?;
    let scope = TenantScopeService::from_session(&user)?;
    let labels = fetch_labels(conn, scope).await?;
    Ok(Json(labels))
}

pub async fn create_label(
    State(_state): State<AppState>,
    tx: Tx,
    jar: CookieJar,
    Json(payload): Json<CreateLabelRequest>,
) -> Result<(StatusCode, Json<LabelResponse>), (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;
    let user = require_authenticated_user(conn, &jar).await?;
    if user.account_type == crate::types::AccountType::Company {
        authorization::require_company_role(&user, authorization::MANAGE_LABELS).await?;
    }
    let scope = TenantScopeService::from_session(&user)?;
    let name = label_service::normalize_label_name(&payload.name)?;
    let color = label_service::normalize_hex_color(&payload.color)?;

    let row = match scope {
        TenantScope::Company { company_id, .. } => sqlx::query_as::<_, Label>(
            r#"
            INSERT INTO labels (id, name, color, company_id, user_id)
            VALUES ($1, $2, $3, $4, NULL)
            RETURNING id, name, color, company_id, user_id, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .bind(color)
        .bind(company_id)
        .fetch_one(&mut **conn)
        .await
        .map_err(map_insert_error)?,
        TenantScope::Personal { user_id } => sqlx::query_as::<_, Label>(
            r#"
            INSERT INTO labels (id, name, color, company_id, user_id)
            VALUES ($1, $2, $3, NULL, $4)
            RETURNING id, name, color, company_id, user_id, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .bind(color)
        .bind(user_id)
        .fetch_one(&mut **conn)
        .await
        .map_err(map_insert_error)?,
    };

    Ok((StatusCode::CREATED, Json(map_label_response(row))))
}

pub async fn update_label(
    tx: Tx,
    jar: CookieJar,
    Path(label_id): Path<Uuid>,
    Json(payload): Json<UpdateLabelRequest>,
) -> Result<Json<LabelResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;
    let user = require_authenticated_user(conn, &jar).await?;
    if user.account_type == crate::types::AccountType::Company {
        authorization::require_company_role(&user, authorization::MANAGE_LABELS).await?;
    }
    let scope = TenantScopeService::from_session(&user)?;
    let name = label_service::normalize_label_name(&payload.name)?;
    let color = label_service::normalize_hex_color(&payload.color)?;

    let row = match scope {
        TenantScope::Company { company_id, .. } => sqlx::query_as::<_, Label>(
            r#"
            UPDATE labels
            SET name = $1, color = $2
            WHERE id = $3 AND company_id = $4
            RETURNING id, name, color, company_id, user_id, created_at
            "#,
        )
        .bind(name)
        .bind(color)
        .bind(label_id)
        .bind(company_id)
        .fetch_optional(&mut **conn)
        .await
        .map_err(|_| internal_error())?,
        TenantScope::Personal { user_id } => sqlx::query_as::<_, Label>(
            r#"
            UPDATE labels
            SET name = $1, color = $2
            WHERE id = $3 AND user_id = $4
            RETURNING id, name, color, company_id, user_id, created_at
            "#,
        )
        .bind(name)
        .bind(color)
        .bind(label_id)
        .bind(user_id)
        .fetch_optional(&mut **conn)
        .await
        .map_err(|_| internal_error())?,
    };

    let Some(row) = row else {
        return Err(not_found("Label not found."));
    };

    Ok(Json(map_label_response(row)))
}

pub async fn delete_label(
    tx: Tx,
    jar: CookieJar,
    Path(label_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;
    let user = require_authenticated_user(conn, &jar).await?;
    if user.account_type == crate::types::AccountType::Company {
        authorization::require_company_role(&user, authorization::MANAGE_LABELS).await?;
    }
    let scope = TenantScopeService::from_session(&user)?;

    let deleted = match scope {
        TenantScope::Company { company_id, .. } => sqlx::query("DELETE FROM labels WHERE id = $1 AND company_id = $2")
            .bind(label_id)
            .bind(company_id)
            .execute(&mut **conn)
            .await
            .map_err(|_| internal_error())?,
        TenantScope::Personal { user_id } => sqlx::query("DELETE FROM labels WHERE id = $1 AND user_id = $2")
            .bind(label_id)
            .bind(user_id)
            .execute(&mut **conn)
            .await
            .map_err(|_| internal_error())?,
    };

    if deleted.rows_affected() == 0 {
        return Err(not_found("Label not found."));
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn fetch_labels(
    conn: &mut PgConnection,
    scope: TenantScope,
) -> Result<Vec<LabelResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let rows = match scope {
        TenantScope::Company { company_id, .. } => sqlx::query_as::<_, Label>(
            r#"
            SELECT id, name, color, company_id, user_id, created_at
            FROM labels
            WHERE company_id = $1
            ORDER BY lower(name) ASC
            "#,
        )
        .bind(company_id)
        .fetch_all(&mut *conn)
        .await
        .map_err(|_| internal_error())?,
        TenantScope::Personal { user_id } => sqlx::query_as::<_, Label>(
            r#"
            SELECT id, name, color, company_id, user_id, created_at
            FROM labels
            WHERE user_id = $1
            ORDER BY lower(name) ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&mut *conn)
        .await
        .map_err(|_| internal_error())?,
    };

    Ok(rows.into_iter().map(map_label_response).collect())
}

fn map_label_response(label: Label) -> LabelResponse {
    LabelResponse {
        id: label.id,
        name: label.name,
        color: label.color,
        created_at: label.created_at,
    }
}

fn map_insert_error(error: sqlx::Error) -> (StatusCode, Json<ApiErrorBody>) {
    if let sqlx::Error::Database(db_error) = &error {
        if db_error.constraint() == Some("labels_unique_company_name")
            || db_error.constraint() == Some("labels_unique_user_name")
        {
            return bad_request("Duplicate label name within this workspace.");
        }
    }
    internal_error()
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

fn internal_error() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiErrorBody {
            message: "Something went wrong. Please try again.".into(),
            ..Default::default()
        }),
    )
}
