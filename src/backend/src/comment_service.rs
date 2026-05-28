use axum::http::StatusCode;
use axum::Json;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::types::{ApiErrorBody, CommentResponse, TenantScope};

pub const MAX_COMMENT_CONTENT_LEN: usize = 10000;

pub fn normalize_comment_content(raw: &str) -> Result<String, (StatusCode, Json<ApiErrorBody>)> {
    let value = raw.trim();
    if value.is_empty() {
        return Err(bad_request("Comment content is required."));
    }
    if value.len() > MAX_COMMENT_CONTENT_LEN {
        return Err(bad_request("Comment content is too long."));
    }
    Ok(value.to_string())
}

pub async fn ensure_task_visible(
    conn: &mut PgConnection,
    scope: TenantScope,
    project_id: Uuid,
    feature_id: Uuid,
    task_id: Uuid,
) -> Result<(), (StatusCode, Json<ApiErrorBody>)> {
    let found: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT t.id
        FROM tasks t
        INNER JOIN features f ON f.id = t.feature_id
        INNER JOIN projects p ON p.id = f.project_id
        WHERE t.id = $1
          AND t.feature_id = $2
          AND f.project_id = $3
          AND (
            ($5::uuid IS NOT NULL AND p.owner_user_id = $5 AND p.company_id IS NULL)
            OR
            ($4::uuid IS NOT NULL AND p.company_id = $4 AND (
                $6::boolean
                OR EXISTS (
                    SELECT 1 FROM project_memberships pm
                    WHERE pm.project_id = p.id AND pm.user_id = $7
                )
            ))
          )
        "#,
    )
    .bind(task_id)
    .bind(feature_id)
    .bind(project_id)
    .bind(scope.company_id_or_null())
    .bind(scope.owner_user_id_or_null())
    .bind(scope.is_company_admin())
    .bind(scope.session_user_id())
    .fetch_optional(&mut *conn)
    .await
    .map_err(|_| internal_error())?;

    if found.is_some() {
        Ok(())
    } else {
        Err(not_found("Task not found."))
    }
}

pub async fn list_comments_for_task(
    conn: &mut PgConnection,
    task_id: Uuid,
) -> Result<Vec<CommentResponse>, (StatusCode, Json<ApiErrorBody>)> {
    sqlx::query_as::<_, CommentResponse>(
        r#"
        SELECT
            c.id,
            c.task_id,
            c.user_id,
            u.fullname AS author_fullname,
            u.avatar_url AS author_avatar_url,
            c.content,
            c.created_at,
            c.updated_at
        FROM task_comments c
        INNER JOIN users u ON u.id = c.user_id
        WHERE c.task_id = $1
        ORDER BY c.created_at ASC
        "#,
    )
    .bind(task_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(|_| internal_error())
}

pub async fn create_comment(
    conn: &mut PgConnection,
    task_id: Uuid,
    user_id: Uuid,
    content: String,
) -> Result<CommentResponse, (StatusCode, Json<ApiErrorBody>)> {
    sqlx::query_as::<_, CommentResponse>(
        r#"
        INSERT INTO task_comments (id, task_id, user_id, content)
        VALUES ($1, $2, $3, $4)
        RETURNING
            id,
            task_id,
            user_id,
            (SELECT fullname FROM users WHERE id = user_id) AS author_fullname,
            (SELECT avatar_url FROM users WHERE id = user_id) AS author_avatar_url,
            content,
            created_at,
            updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(task_id)
    .bind(user_id)
    .bind(content)
    .fetch_one(&mut *conn)
    .await
    .map_err(|_| internal_error())
}

pub async fn update_comment(
    conn: &mut PgConnection,
    scope: TenantScope,
    project_id: Uuid,
    feature_id: Uuid,
    task_id: Uuid,
    comment_id: Uuid,
    content: String,
) -> Result<CommentResponse, (StatusCode, Json<ApiErrorBody>)> {
    let owner_user_id = fetch_comment_owner(conn, scope, project_id, feature_id, task_id, comment_id).await?;
    if owner_user_id != scope.session_user_id() && !scope.is_company_admin() {
        return Err(forbidden("You do not have permission to modify this comment."));
    }

    let updated = sqlx::query_as::<_, CommentResponse>(
        r#"
        UPDATE task_comments c
        SET content = $1,
            updated_at = NOW()
        FROM tasks t
        INNER JOIN features f ON f.id = t.feature_id
        INNER JOIN projects p ON p.id = f.project_id
        WHERE c.id = $2
          AND c.task_id = $3
          AND t.id = c.task_id
          AND t.id = $3
          AND t.feature_id = $4
          AND f.project_id = $5
          AND (
            ($7::uuid IS NOT NULL AND p.owner_user_id = $7 AND p.company_id IS NULL)
            OR
            ($6::uuid IS NOT NULL AND p.company_id = $6 AND (
                $8::boolean
                OR EXISTS (
                    SELECT 1 FROM project_memberships pm
                    WHERE pm.project_id = p.id AND pm.user_id = $9
                )
            ))
          )
        RETURNING
            c.id,
            c.task_id,
            c.user_id,
            (SELECT fullname FROM users WHERE id = c.user_id) AS author_fullname,
            (SELECT avatar_url FROM users WHERE id = c.user_id) AS author_avatar_url,
            c.content,
            c.created_at,
            c.updated_at
        "#,
    )
    .bind(content)
    .bind(comment_id)
    .bind(task_id)
    .bind(feature_id)
    .bind(project_id)
    .bind(scope.company_id_or_null())
    .bind(scope.owner_user_id_or_null())
    .bind(scope.is_company_admin())
    .bind(scope.session_user_id())
    .fetch_optional(&mut *conn)
    .await
    .map_err(|_| internal_error())?;

    updated.ok_or_else(|| not_found("Comment not found."))
}

pub async fn delete_comment(
    conn: &mut PgConnection,
    scope: TenantScope,
    project_id: Uuid,
    feature_id: Uuid,
    task_id: Uuid,
    comment_id: Uuid,
) -> Result<(), (StatusCode, Json<ApiErrorBody>)> {
    let owner_user_id = fetch_comment_owner(conn, scope, project_id, feature_id, task_id, comment_id).await?;
    if owner_user_id != scope.session_user_id() && !scope.is_company_admin() {
        return Err(forbidden("You do not have permission to modify this comment."));
    }

    let deleted = sqlx::query(
        r#"
        DELETE FROM task_comments c
        USING tasks t
        INNER JOIN features f ON f.id = t.feature_id
        INNER JOIN projects p ON p.id = f.project_id
        WHERE c.id = $1
          AND c.task_id = $2
          AND t.id = c.task_id
          AND t.id = $2
          AND t.feature_id = $3
          AND f.project_id = $4
          AND (
            ($6::uuid IS NOT NULL AND p.owner_user_id = $6 AND p.company_id IS NULL)
            OR
            ($5::uuid IS NOT NULL AND p.company_id = $5 AND (
                $7::boolean
                OR EXISTS (
                    SELECT 1 FROM project_memberships pm
                    WHERE pm.project_id = p.id AND pm.user_id = $8
                )
            ))
          )
        "#,
    )
    .bind(comment_id)
    .bind(task_id)
    .bind(feature_id)
    .bind(project_id)
    .bind(scope.company_id_or_null())
    .bind(scope.owner_user_id_or_null())
    .bind(scope.is_company_admin())
    .bind(scope.session_user_id())
    .execute(&mut *conn)
    .await
    .map_err(|_| internal_error())?;

    if deleted.rows_affected() == 0 {
        return Err(not_found("Comment not found."));
    }
    Ok(())
}

async fn fetch_comment_owner(
    conn: &mut PgConnection,
    scope: TenantScope,
    project_id: Uuid,
    feature_id: Uuid,
    task_id: Uuid,
    comment_id: Uuid,
) -> Result<Uuid, (StatusCode, Json<ApiErrorBody>)> {
    let owner = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT c.user_id
        FROM task_comments c
        INNER JOIN tasks t ON t.id = c.task_id
        INNER JOIN features f ON f.id = t.feature_id
        INNER JOIN projects p ON p.id = f.project_id
        WHERE c.id = $1
          AND c.task_id = $2
          AND t.feature_id = $3
          AND f.project_id = $4
          AND (
            ($6::uuid IS NOT NULL AND p.owner_user_id = $6 AND p.company_id IS NULL)
            OR
            ($5::uuid IS NOT NULL AND p.company_id = $5 AND (
                $7::boolean
                OR EXISTS (
                    SELECT 1 FROM project_memberships pm
                    WHERE pm.project_id = p.id AND pm.user_id = $8
                )
            ))
          )
        "#,
    )
    .bind(comment_id)
    .bind(task_id)
    .bind(feature_id)
    .bind(project_id)
    .bind(scope.company_id_or_null())
    .bind(scope.owner_user_id_or_null())
    .bind(scope.is_company_admin())
    .bind(scope.session_user_id())
    .fetch_optional(&mut *conn)
    .await
    .map_err(|_| internal_error())?;

    owner.ok_or_else(|| not_found("Comment not found."))
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

fn forbidden(message: impl Into<String>) -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::FORBIDDEN,
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
