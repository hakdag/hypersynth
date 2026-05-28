use axum::extract::Path;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use uuid::Uuid;

use crate::auth_route::require_authenticated_user;
use crate::comment_service;
use crate::tenant_scope_service::TenantScopeService;
use crate::tx_extractor::missing_tx_error;
use crate::types::{ApiErrorBody, CommentResponse, CreateCommentRequest, Tx, UpdateCommentRequest};

pub async fn list_task_comments(
    tx: Tx,
    jar: CookieJar,
    Path((project_id, feature_id, task_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<Vec<CommentResponse>>, (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;
    let user = require_authenticated_user(conn, &jar).await?;
    let scope = TenantScopeService::from_session(&user)?;
    comment_service::ensure_task_visible(conn, scope, project_id, feature_id, task_id).await?;
    let comments = comment_service::list_comments_for_task(conn, task_id).await?;
    Ok(Json(comments))
}

pub async fn create_task_comment(
    tx: Tx,
    jar: CookieJar,
    Path((project_id, feature_id, task_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<CommentResponse>), (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;
    let user = require_authenticated_user(conn, &jar).await?;
    let scope = TenantScopeService::from_session(&user)?;
    comment_service::ensure_task_visible(conn, scope, project_id, feature_id, task_id).await?;
    let content = comment_service::normalize_comment_content(&payload.content)?;
    let comment = comment_service::create_comment(conn, task_id, scope.session_user_id(), content).await?;
    Ok((StatusCode::CREATED, Json(comment)))
}

pub async fn update_task_comment(
    tx: Tx,
    jar: CookieJar,
    Path((project_id, feature_id, task_id, comment_id)): Path<(Uuid, Uuid, Uuid, Uuid)>,
    Json(payload): Json<UpdateCommentRequest>,
) -> Result<Json<CommentResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;
    let user = require_authenticated_user(conn, &jar).await?;
    let scope = TenantScopeService::from_session(&user)?;
    comment_service::ensure_task_visible(conn, scope, project_id, feature_id, task_id).await?;
    let content = comment_service::normalize_comment_content(&payload.content)?;
    let comment = comment_service::update_comment(
        conn,
        scope,
        project_id,
        feature_id,
        task_id,
        comment_id,
        content,
    )
    .await?;
    Ok(Json(comment))
}

pub async fn delete_task_comment(
    tx: Tx,
    jar: CookieJar,
    Path((project_id, feature_id, task_id, comment_id)): Path<(Uuid, Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;
    let user = require_authenticated_user(conn, &jar).await?;
    let scope = TenantScopeService::from_session(&user)?;
    comment_service::ensure_task_visible(conn, scope, project_id, feature_id, task_id).await?;
    comment_service::delete_comment(conn, scope, project_id, feature_id, task_id, comment_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
