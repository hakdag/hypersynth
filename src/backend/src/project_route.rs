use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use tracing::{info, warn};

use crate::app_state::AppState;
use crate::auth_route;
use crate::types::{ApiErrorBody, CreateProjectRequest, ProjectDetailResponse, ProjectResponse};
use uuid::Uuid;

pub async fn list_projects(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<ProjectResponse>>, (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    info!(has_cookie, "api: GET /api/v1/projects");

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                "api: GET /api/v1/projects -> auth error response"
            );
            (status, json)
        })?;

    let rows = sqlx::query_as::<_, ProjectResponse>(
        r#"
        SELECT id, user_id, name, requirements, status, created_at
        FROM projects
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        warn!(error = %e, "list_projects: query failed");
        internal_error()
    })?;

    info!(
        user_id = %user.id,
        row_count = rows.len(),
        "api: GET /api/v1/projects -> 200 OK"
    );

    Ok(Json(rows))
}

pub async fn get_project(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectDetailResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    info!(has_cookie, %project_id, "api: GET /api/v1/projects/:id");

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                "api: GET /api/v1/projects/:id -> auth error response"
            );
            (status, json)
        })?;

    let row = sqlx::query_as::<_, ProjectDetailResponse>(
        r#"
        SELECT
            id,
            user_id,
            name,
            requirements,
            status,
            created_at,
            (ai_api_key IS NOT NULL AND btrim(ai_api_key) <> '') AS has_ai_api_key
        FROM projects
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(project_id)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        warn!(error = %e, "get_project: query failed");
        internal_error()
    })?;

    let Some(row) = row else {
        warn!(
            user_id = %user.id,
            project_id = %project_id,
            "api: GET /api/v1/projects/:id -> 404"
        );
        return Err(not_found("Project not found."));
    };

    info!(
        user_id = %user.id,
        project_id = %row.id,
        "api: GET /api/v1/projects/:id -> 200 OK"
    );

    Ok(Json(row))
}

pub async fn create_project(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    let name_len = payload.name.len();
    let has_requirements = payload
        .requirements
        .as_ref()
        .is_some_and(|s| !s.trim().is_empty());
    let has_ai_key = payload
        .ai_api_key
        .as_ref()
        .is_some_and(|s| !s.trim().is_empty());
    info!(
        has_cookie,
        name_len,
        has_requirements,
        has_ai_key,
        "api: POST /api/v1/projects (body summarized; API key not logged)"
    );

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                "api: POST /api/v1/projects -> auth error response"
            );
            (status, json)
        })?;

    let name = payload.name.trim();
    if name.is_empty() {
        warn!(user_id = %user.id, "api: POST /api/v1/projects -> 400 empty name");
        return Err(bad_request("Project name is required."));
    }

    let requirements = payload
        .requirements
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let ai_key = payload
        .ai_api_key
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let row = sqlx::query_as::<_, ProjectResponse>(
        r#"
        INSERT INTO projects (user_id, name, requirements, status, ai_api_key)
        VALUES ($1, $2, $3, 'Pending', $4)
        RETURNING id, user_id, name, requirements, status, created_at
        "#,
    )
    .bind(user.id)
    .bind(name)
    .bind(requirements.as_ref())
    .bind(ai_key.as_ref())
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        warn!(error = %e, user_id = %user.id, "create_project: insert failed");
        internal_error()
    })?;

    info!(
        project_id = %row.id,
        user_id = %user.id,
        project_status = %row.status,
        "api: POST /api/v1/projects -> 201 CREATED"
    );

    Ok((StatusCode::CREATED, Json(row)))
}

fn not_found(message: impl Into<String>) -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiErrorBody {
            message: message.into(),
        }),
    )
}

fn bad_request(message: impl Into<String>) -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiErrorBody {
            message: message.into(),
        }),
    )
}

fn internal_error() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiErrorBody {
            message: "Something went wrong. Please try again.".into(),
        }),
    )
}
