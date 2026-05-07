use std::path::Path as FsPath;

use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use serde_json::json;
use tracing::{info, warn};

use crate::app_state::AppState;
use crate::auth_route;
use crate::types::{
    ApiErrorBody, CreateFeatureRequest, CreateProjectRequest, CreateTaskRequest, FeatureResponse,
    ProjectDetailResponse, ProjectDocumentResponse, ProjectResponse, TaskDetailResponse,
    TaskResponse, UpdateFeatureRequest, UpdateProjectRequest, UpdateTaskRequest,
};
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

pub async fn update_project(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    let name_len = payload.name.len();
    info!(
        has_cookie,
        %project_id,
        name_len,
        clear_ai_api_key = payload.clear_ai_api_key,
        has_new_ai_key = payload.ai_api_key.as_ref().is_some_and(|s| !s.trim().is_empty()),
        "api: PATCH /api/v1/projects/:id (body summarized; API key not logged)"
    );

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                "api: PATCH /api/v1/projects/:id -> auth error response"
            );
            (status, json)
        })?;

    let name = payload.name.trim();
    if name.is_empty() {
        warn!(user_id = %user.id, %project_id, "api: PATCH /api/v1/projects/:id -> 400 empty name");
        return Err(bad_request("Project name is required."));
    }

    let status = payload.status.trim();
    if status != "Pending" && status != "In Progress" && status != "Done" {
        warn!(user_id = %user.id, %project_id, "api: PATCH /api/v1/projects/:id -> 400 invalid status");
        return Err(bad_request(
            "Status must be one of: Pending, In Progress, Done.",
        ));
    }

    let requirements_trimmed = payload.requirements.trim();
    let requirements_for_db = if requirements_trimmed.is_empty() {
        None
    } else {
        Some(requirements_trimmed.to_string())
    };

    let row = sqlx::query_as::<_, ProjectResponse>(
        r#"
        UPDATE projects
        SET
            name = $1,
            requirements = $2,
            status = $3,
            ai_api_key = CASE
                WHEN $4::boolean THEN NULL
                WHEN $5 IS NOT NULL AND btrim($5) <> '' THEN btrim($5)
                ELSE ai_api_key
            END
        WHERE id = $6 AND user_id = $7
        RETURNING id, user_id, name, requirements, status, created_at
        "#,
    )
    .bind(name)
    .bind(requirements_for_db.as_ref())
    .bind(status)
    .bind(payload.clear_ai_api_key)
    .bind(payload.ai_api_key.as_ref())
    .bind(project_id)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        warn!(error = %e, user_id = %user.id, %project_id, "update_project: update failed");
        internal_error()
    })?;

    let Some(row) = row else {
        warn!(
            user_id = %user.id,
            %project_id,
            "api: PATCH /api/v1/projects/:id -> 404 not owner or missing"
        );
        return Err(not_found("Project not found."));
    };

    info!(
        user_id = %user.id,
        project_id = %row.id,
        project_status = %row.status,
        "api: PATCH /api/v1/projects/:id -> 200 OK"
    );

    Ok(Json(row))
}

pub async fn create_feature(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<CreateFeatureRequest>,
) -> Result<(StatusCode, Json<FeatureResponse>), (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    let title_len = payload.title.len();
    info!(
        has_cookie,
        %project_id,
        title_len,
        "api: POST /api/v1/projects/:id/features (body summarized)"
    );

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                "api: POST /api/v1/projects/:id/features -> auth error response"
            );
            (status, json)
        })?;

    let title = payload.title.trim();
    if title.is_empty() {
        warn!(
            user_id = %user.id,
            %project_id,
            "api: POST /api/v1/projects/:id/features -> 400 empty title"
        );
        return Err(bad_request("Feature title is required."));
    }

    let requirements = payload
        .requirements
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let ok: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT id
        FROM projects
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(project_id)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        warn!(error = %e, "create_feature: project lookup failed");
        internal_error()
    })?;

    if ok.is_none() {
        warn!(
            user_id = %user.id,
            %project_id,
            "api: POST /api/v1/projects/:id/features -> 404 project"
        );
        return Err(not_found("Project not found."));
    }

    let row = sqlx::query_as::<_, FeatureResponse>(
        r#"
        INSERT INTO features (project_id, title, requirements, status)
        VALUES ($1, $2, $3, 'Pending')
        RETURNING id, project_id, title, requirements, status, created_at
        "#,
    )
    .bind(project_id)
    .bind(title)
    .bind(requirements.as_ref())
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        warn!(error = %e, user_id = %user.id, %project_id, "create_feature: insert failed");
        internal_error()
    })?;

    info!(
        feature_id = %row.id,
        user_id = %user.id,
        project_id = %project_id,
        feature_status = %row.status,
        "api: POST /api/v1/projects/:id/features -> 201 CREATED"
    );

    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn list_project_features(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<FeatureResponse>>, (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    info!(has_cookie, %project_id, "api: GET /api/v1/projects/:id/features");

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                "api: GET /api/v1/projects/:id/features -> auth error response"
            );
            (status, json)
        })?;

    let rows = sqlx::query_as::<_, FeatureResponse>(
        r#"
        SELECT f.id, f.project_id, f.title, f.requirements, f.status, f.created_at
        FROM features f
        INNER JOIN projects p ON p.id = f.project_id
        WHERE f.project_id = $1 AND p.user_id = $2
        ORDER BY f.created_at DESC
        "#,
    )
    .bind(project_id)
    .bind(user.id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        warn!(error = %e, user_id = %user.id, %project_id, "list_project_features: query failed");
        internal_error()
    })?;

    info!(
        user_id = %user.id,
        %project_id,
        row_count = rows.len(),
        "api: GET /api/v1/projects/:id/features -> 200 OK"
    );

    Ok(Json(rows))
}

pub async fn list_project_documents(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<ProjectDocumentResponse>>, (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    info!(has_cookie, %project_id, "api: GET /api/v1/projects/:id/documents");

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                "api: GET /api/v1/projects/:id/documents -> auth error response"
            );
            (status, json)
        })?;

    let rows = sqlx::query_as::<_, ProjectDocumentResponse>(
        r#"
        SELECT d.id, d.project_id, d.file_path, d.metadata, d.created_at
        FROM project_documents d
        INNER JOIN projects p ON p.id = d.project_id
        WHERE d.project_id = $1 AND p.user_id = $2
        ORDER BY d.created_at DESC
        "#,
    )
    .bind(project_id)
    .bind(user.id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        warn!(error = %e, user_id = %user.id, %project_id, "list_project_documents: query failed");
        internal_error()
    })?;

    info!(
        user_id = %user.id,
        %project_id,
        row_count = rows.len(),
        "api: GET /api/v1/projects/:id/documents -> 200 OK"
    );

    Ok(Json(rows))
}

pub async fn upload_project_documents(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(project_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Vec<ProjectDocumentResponse>>), (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    info!(has_cookie, %project_id, "api: POST /api/v1/projects/:id/documents");

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                "api: POST /api/v1/projects/:id/documents -> auth error response"
            );
            (status, json)
        })?;

    // this part will be changed when we introduce company-scoped documents
    let ok: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT id
        FROM projects
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(project_id)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        warn!(error = %e, user_id = %user.id, %project_id, "upload_project_documents: project lookup failed");
        internal_error()
    })?;

    if ok.is_none() {
        warn!(
            user_id = %user.id,
            %project_id,
            "api: POST /api/v1/projects/:id/documents -> 404 project"
        );
        return Err(not_found("Project not found."));
    }

    // this part will be changed when we introduce company-scoped documents
    let upload_root = FsPath::new(&state.document_upload_dir);
    let project_dir = upload_root
        .join(user.id.to_string())
        .join(project_id.to_string());

    tokio::fs::create_dir_all(&project_dir).await.map_err(|e| {
        warn!(
            error = %e,
            path = %project_dir.display(),
            user_id = %user.id,
            %project_id,
            "upload_project_documents: create upload directory failed"
        );
        internal_error()
    })?;

    let mut rows = Vec::new();
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        warn!(error = %e, user_id = %user.id, %project_id, "upload_project_documents: multipart read failed");
        bad_request("Could not read the uploaded file.")
    })? {
        let Some(original_name) = field.file_name().map(|name| name.to_string()) else {
            continue;
        };

        let content_type = field.content_type().map(|value| value.to_string());
        if !is_allowed_document_name(&original_name) {
            warn!(
                user_id = %user.id,
                %project_id,
                original_name = %original_name,
                "api: POST /api/v1/projects/:id/documents -> 400 unsupported file type"
            );
            return Err(bad_request(
                "Unsupported file type. Upload Markdown, text, Excel, Word, or common image files.",
            ));
        }

        let safe_name = safe_document_file_name(&original_name);
        let extension = file_extension(&safe_name);
        let stored_name = if extension.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            format!("{}.{}", Uuid::new_v4(), extension)
        };
        let target_path = project_dir.join(stored_name);
        let bytes = field.bytes().await.map_err(|e| {
            warn!(error = %e, user_id = %user.id, %project_id, "upload_project_documents: file bytes read failed");
            bad_request("Could not read the uploaded file.")
        })?;

        if bytes.is_empty() {
            warn!(
                user_id = %user.id,
                %project_id,
                original_name = %original_name,
                "api: POST /api/v1/projects/:id/documents -> 400 empty file"
            );
            return Err(bad_request("Uploaded files must not be empty."));
        }

        tokio::fs::write(&target_path, &bytes).await.map_err(|e| {
            warn!(
                error = %e,
                path = %target_path.display(),
                user_id = %user.id,
                %project_id,
                "upload_project_documents: file write failed"
            );
            internal_error()
        })?;

        let file_path = target_path.to_string_lossy().into_owned();
        let metadata = json!({
            "originalFilename": original_name,
            "storedFilename": safe_name,
            "size": bytes.len(),
            "contentType": content_type,
        });

        let row = sqlx::query_as::<_, ProjectDocumentResponse>(
            r#"
            INSERT INTO project_documents (project_id, file_path, metadata)
            VALUES ($1, $2, $3)
            RETURNING id, project_id, file_path, metadata, created_at
            "#,
        )
        .bind(project_id)
        .bind(&file_path)
        .bind(metadata)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            warn!(
                error = %e,
                path = %file_path,
                user_id = %user.id,
                %project_id,
                "upload_project_documents: metadata insert failed"
            );
            internal_error()
        })?;

        rows.push(row);
    }

    if rows.is_empty() {
        warn!(
            user_id = %user.id,
            %project_id,
            "api: POST /api/v1/projects/:id/documents -> 400 no files"
        );
        return Err(bad_request("Select at least one file to upload."));
    }

    info!(
        user_id = %user.id,
        %project_id,
        document_count = rows.len(),
        "api: POST /api/v1/projects/:id/documents -> 201 CREATED"
    );

    Ok((StatusCode::CREATED, Json(rows)))
}

pub async fn get_project_feature(
    State(state): State<AppState>,
    jar: CookieJar,
    Path((project_id, feature_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<FeatureResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    info!(has_cookie, %project_id, %feature_id, "api: GET /api/v1/projects/:id/features/:feature_id");

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                "api: GET /api/v1/projects/:id/features/:feature_id -> auth error response"
            );
            (status, json)
        })?;

    let row = sqlx::query_as::<_, FeatureResponse>(
        r#"
        SELECT f.id, f.project_id, f.title, f.requirements, f.status, f.created_at
        FROM features f
        INNER JOIN projects p ON p.id = f.project_id
        WHERE f.id = $1 AND f.project_id = $2 AND p.user_id = $3
        "#,
    )
    .bind(feature_id)
    .bind(project_id)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        warn!(error = %e, user_id = %user.id, %project_id, %feature_id, "get_project_feature: query failed");
        internal_error()
    })?;

    let Some(row) = row else {
        warn!(
            user_id = %user.id,
            %project_id,
            %feature_id,
            "api: GET /api/v1/projects/:id/features/:feature_id -> 404"
        );
        return Err(not_found("Feature not found."));
    };

    info!(
        user_id = %user.id,
        %project_id,
        feature_id = %row.id,
        "api: GET /api/v1/projects/:id/features/:feature_id -> 200 OK"
    );

    Ok(Json(row))
}

pub async fn list_feature_tasks(
    State(state): State<AppState>,
    jar: CookieJar,
    Path((project_id, feature_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<TaskResponse>>, (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    info!(
        has_cookie,
        %project_id,
        %feature_id,
        "api: GET /api/v1/projects/:id/features/:feature_id/tasks"
    );

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                "api: GET /api/v1/projects/:id/features/:feature_id/tasks -> auth error response"
            );
            (status, json)
        })?;

    let rows = sqlx::query_as::<_, TaskResponse>(
        r#"
        SELECT
            t.id,
            t.feature_id,
            t.title,
            t.description,
            t.status,
            t.created_by,
            t.created_at,
            t.priority,
            t.assignee_user_id,
            au.fullname AS assignee_fullname,
            au.avatar_url AS assignee_avatar_url,
            cu.fullname AS creator_fullname,
            cu.avatar_url AS creator_avatar_url
        FROM tasks t
        INNER JOIN features f ON f.id = t.feature_id
        INNER JOIN projects p ON p.id = f.project_id
        LEFT JOIN users au ON au.id = t.assignee_user_id
        LEFT JOIN users cu ON cu.id = t.creator_user_id
        WHERE t.feature_id = $1 AND f.project_id = $2 AND p.user_id = $3
        ORDER BY t.created_at DESC
        "#,
    )
    .bind(feature_id)
    .bind(project_id)
    .bind(user.id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        warn!(
            error = %e,
            user_id = %user.id,
            %project_id,
            %feature_id,
            "list_feature_tasks: query failed"
        );
        internal_error()
    })?;

    info!(
        user_id = %user.id,
        %project_id,
        %feature_id,
        row_count = rows.len(),
        "api: GET /api/v1/projects/:id/features/:feature_id/tasks -> 200 OK"
    );

    Ok(Json(rows))
}

pub async fn get_project_task(
    State(state): State<AppState>,
    jar: CookieJar,
    Path((project_id, feature_id, task_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<TaskDetailResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    info!(
        has_cookie,
        %project_id,
        %feature_id,
        %task_id,
        "api: GET /api/v1/projects/:id/features/:feature_id/tasks/:task_id"
    );

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                "api: GET /api/v1/projects/:id/features/:feature_id/tasks/:task_id -> auth error response"
            );
            (status, json)
        })?;

    let row = sqlx::query_as::<_, TaskDetailResponse>(
        r#"
        SELECT
            t.id,
            t.feature_id,
            t.title,
            t.description,
            t.status,
            t.created_by,
            t.created_at,
            t.priority,
            t.assignee_user_id,
            au.fullname AS assignee_fullname,
            au.avatar_url AS assignee_avatar_url,
            cu.fullname AS creator_fullname,
            cu.avatar_url AS creator_avatar_url,
            f.title AS feature_title,
            p.id AS project_id,
            p.name AS project_name
        FROM tasks t
        INNER JOIN features f ON f.id = t.feature_id
        INNER JOIN projects p ON p.id = f.project_id
        LEFT JOIN users au ON au.id = t.assignee_user_id
        LEFT JOIN users cu ON cu.id = t.creator_user_id
        WHERE t.id = $1 AND t.feature_id = $2 AND f.project_id = $3 AND p.user_id = $4
        "#,
    )
    .bind(task_id)
    .bind(feature_id)
    .bind(project_id)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        warn!(
            error = %e,
            user_id = %user.id,
            %project_id,
            %feature_id,
            %task_id,
            "get_project_task: query failed"
        );
        internal_error()
    })?;

    let Some(row) = row else {
        warn!(
            user_id = %user.id,
            %project_id,
            %feature_id,
            %task_id,
            "api: GET /api/v1/projects/:id/features/:feature_id/tasks/:task_id -> 404"
        );
        return Err(not_found("Task not found."));
    };

    info!(
        user_id = %user.id,
        %project_id,
        %feature_id,
        task_id = %row.id,
        "api: GET /api/v1/projects/:id/features/:feature_id/tasks/:task_id -> 200 OK"
    );

    Ok(Json(row))
}

pub async fn create_task(
    State(state): State<AppState>,
    jar: CookieJar,
    Path((project_id, feature_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<TaskResponse>), (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    let title_len = payload.title.len();
    info!(
        has_cookie,
        %project_id,
        %feature_id,
        title_len,
        "api: POST /api/v1/projects/:id/features/:feature_id/tasks (body summarized)"
    );

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                "api: POST /api/v1/projects/:id/features/:feature_id/tasks -> auth error response"
            );
            (status, json)
        })?;

    let title = payload.title.trim();
    if title.is_empty() {
        warn!(
            user_id = %user.id,
            %project_id,
            %feature_id,
            "api: POST /api/v1/projects/:id/features/:feature_id/tasks -> 400 empty title"
        );
        return Err(bad_request("Task title is required."));
    }

    let description = payload
        .description
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let priority_raw = payload
        .priority
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Standard");
    let priority_val = match priority_raw {
        "Standard" | "Elevated" | "Critical" => priority_raw,
        _ => {
            warn!(
                user_id = %user.id,
                %project_id,
                %feature_id,
                "api: POST /api/v1/projects/:id/features/:feature_id/tasks -> 400 invalid priority"
            );
            return Err(bad_request(
                "Priority must be Standard, Elevated, or Critical.",
            ));
        }
    };

    let assignee_bind: Option<Uuid> = if payload.unassigned {
        None
    } else if let Some(id) = payload.assignee_user_id {
        if id != user.id {
            warn!(
                user_id = %user.id,
                %project_id,
                %feature_id,
                "api: POST /api/v1/projects/:id/features/:feature_id/tasks -> 400 foreign assignee"
            );
            return Err(bad_request(
                "You can only assign tasks to yourself in this workspace.",
            ));
        }
        Some(id)
    } else {
        Some(user.id)
    };

    let row = sqlx::query_as::<_, TaskResponse>(
        r#"
        WITH ins AS (
            INSERT INTO tasks (
                feature_id,
                title,
                description,
                status,
                created_by,
                priority,
                assignee_user_id,
                creator_user_id
            )
            SELECT f.id, $4, $5, 'Pending', 'User', $6, $7, $8
            FROM features f
            INNER JOIN projects p ON p.id = f.project_id
            WHERE f.id = $1 AND f.project_id = $2 AND p.user_id = $3
            RETURNING
                id,
                feature_id,
                title,
                description,
                status,
                created_by,
                created_at,
                priority,
                assignee_user_id,
                creator_user_id
        )
        SELECT
            ins.id,
            ins.feature_id,
            ins.title,
            ins.description,
            ins.status,
            ins.created_by,
            ins.created_at,
            ins.priority,
            ins.assignee_user_id,
            au.fullname AS assignee_fullname,
            au.avatar_url AS assignee_avatar_url,
            cu.fullname AS creator_fullname,
            cu.avatar_url AS creator_avatar_url
        FROM ins
        LEFT JOIN users au ON au.id = ins.assignee_user_id
        LEFT JOIN users cu ON cu.id = ins.creator_user_id
        "#,
    )
    .bind(feature_id)
    .bind(project_id)
    .bind(user.id)
    .bind(title)
    .bind(description.as_ref())
    .bind(priority_val)
    .bind(assignee_bind)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        warn!(
            error = %e,
            user_id = %user.id,
            %project_id,
            %feature_id,
            "create_task: insert failed"
        );
        internal_error()
    })?;

    let Some(row) = row else {
        warn!(
            user_id = %user.id,
            %project_id,
            %feature_id,
            "api: POST /api/v1/projects/:id/features/:feature_id/tasks -> 404"
        );
        return Err(not_found("Feature not found."));
    };

    info!(
        task_id = %row.id,
        user_id = %user.id,
        %project_id,
        %feature_id,
        task_status = %row.status,
        task_created_by = %row.created_by,
        "api: POST /api/v1/projects/:id/features/:feature_id/tasks -> 201 CREATED"
    );

    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn update_project_feature(
    State(state): State<AppState>,
    jar: CookieJar,
    Path((project_id, feature_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateFeatureRequest>,
) -> Result<Json<FeatureResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    info!(
        has_cookie,
        %project_id,
        %feature_id,
        "api: PATCH /api/v1/projects/:id/features/:feature_id"
    );

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                "api: PATCH /api/v1/projects/:id/features/:feature_id -> auth error response"
            );
            (status, json)
        })?;

    let title = payload.title.trim();
    if title.is_empty() {
        warn!(
            user_id = %user.id,
            %project_id,
            %feature_id,
            "api: PATCH /api/v1/projects/:id/features/:feature_id -> 400 empty title"
        );
        return Err(bad_request("Feature title is required."));
    }

    let requirements = payload
        .requirements
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let status_trimmed = payload.status.trim();
    if !matches!(status_trimmed, "Pending" | "In Progress" | "Done") {
        warn!(
            user_id = %user.id,
            %project_id,
            %feature_id,
            "api: PATCH /api/v1/projects/:id/features/:feature_id -> 400 bad status"
        );
        return Err(bad_request("Status must be Pending, In Progress, or Done."));
    }

    let row = sqlx::query_as::<_, FeatureResponse>(
        r#"
        UPDATE features f
        SET title = $4,
            requirements = $5,
            status = $6
        FROM projects p
        WHERE f.id = $1
          AND f.project_id = $2
          AND p.id = f.project_id
          AND p.user_id = $3
        RETURNING f.id, f.project_id, f.title, f.requirements, f.status, f.created_at
        "#,
    )
    .bind(feature_id)
    .bind(project_id)
    .bind(user.id)
    .bind(title)
    .bind(requirements.as_ref())
    .bind(status_trimmed)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        warn!(
            error = %e,
            user_id = %user.id,
            %project_id,
            %feature_id,
            "update_project_feature: update failed"
        );
        internal_error()
    })?;

    let Some(row) = row else {
        warn!(
            user_id = %user.id,
            %project_id,
            %feature_id,
            "api: PATCH /api/v1/projects/:id/features/:feature_id -> 404"
        );
        return Err(not_found("Feature not found."));
    };

    info!(
        user_id = %user.id,
        %project_id,
        feature_id = %row.id,
        feature_status = %row.status,
        "api: PATCH /api/v1/projects/:id/features/:feature_id -> 200 OK"
    );

    Ok(Json(row))
}

pub async fn update_project_task(
    State(state): State<AppState>,
    jar: CookieJar,
    Path((project_id, feature_id, task_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<TaskDetailResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let has_cookie = auth_route::has_session_cookie(&jar);
    info!(
        has_cookie,
        %project_id,
        %feature_id,
        %task_id,
        "api: PATCH /api/v1/projects/:id/features/:feature_id/tasks/:task_id"
    );

    let user = auth_route::require_authenticated_user(&state.pool, &jar)
        .await
        .map_err(|(status, json)| {
            warn!(
                status = status.as_u16(),
                message = %json.message,
                has_cookie,
                "api: PATCH /api/v1/projects/:id/features/:feature_id/tasks/:task_id -> auth error response"
            );
            (status, json)
        })?;

    let title = payload.title.trim();
    if title.is_empty() {
        warn!(
            user_id = %user.id,
            %project_id,
            %feature_id,
            %task_id,
            "api: PATCH task -> 400 empty title"
        );
        return Err(bad_request("Task title is required."));
    }

    let description = payload
        .description
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let status_trimmed = payload.status.trim();
    if !matches!(status_trimmed, "Pending" | "In Progress" | "Done") {
        warn!(
            user_id = %user.id,
            %project_id,
            %feature_id,
            %task_id,
            "api: PATCH task -> 400 bad status"
        );
        return Err(bad_request("Status must be Pending, In Progress, or Done."));
    }

    let priority_raw = payload.priority.trim();
    let priority_trimmed = if priority_raw.is_empty() {
        "Standard"
    } else {
        priority_raw
    };
    let priority_val = match priority_trimmed {
        "Standard" | "Elevated" | "Critical" => priority_trimmed,
        _ => {
            warn!(
                user_id = %user.id,
                %project_id,
                %feature_id,
                %task_id,
                "api: PATCH task -> 400 invalid priority"
            );
            return Err(bad_request(
                "Priority must be Standard, Elevated, or Critical.",
            ));
        }
    };

    let assignee_bind: Option<Uuid> = if payload.unassigned {
        None
    } else if let Some(id) = payload.assignee_user_id {
        if id != user.id {
            warn!(
                user_id = %user.id,
                %project_id,
                %feature_id,
                %task_id,
                "api: PATCH task -> 400 foreign assignee"
            );
            return Err(bad_request(
                "You can only assign tasks to yourself in this workspace.",
            ));
        }
        Some(id)
    } else {
        Some(user.id)
    };

    let row = sqlx::query_as::<_, TaskDetailResponse>(
        r#"
        WITH updated AS (
            UPDATE tasks t
            SET title = $5,
                description = $6,
                status = $7,
                priority = $8,
                assignee_user_id = $9
            FROM features f
            INNER JOIN projects p ON p.id = f.project_id
            WHERE t.id = $1
              AND t.feature_id = $2
              AND f.id = $2
              AND f.project_id = $3
              AND p.user_id = $4
            RETURNING t.id
        )
        SELECT
            t.id,
            t.feature_id,
            t.title,
            t.description,
            t.status,
            t.created_by,
            t.created_at,
            t.priority,
            t.assignee_user_id,
            au.fullname AS assignee_fullname,
            au.avatar_url AS assignee_avatar_url,
            cu.fullname AS creator_fullname,
            cu.avatar_url AS creator_avatar_url,
            f.title AS feature_title,
            p.id AS project_id,
            p.name AS project_name
        FROM tasks t
        INNER JOIN updated u ON u.id = t.id
        INNER JOIN features f ON f.id = t.feature_id
        INNER JOIN projects p ON p.id = f.project_id
        LEFT JOIN users au ON au.id = t.assignee_user_id
        LEFT JOIN users cu ON cu.id = t.creator_user_id
        "#,
    )
    .bind(task_id)
    .bind(feature_id)
    .bind(project_id)
    .bind(user.id)
    .bind(title)
    .bind(description.as_ref())
    .bind(status_trimmed)
    .bind(priority_val)
    .bind(assignee_bind)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        warn!(
            error = %e,
            user_id = %user.id,
            %project_id,
            %feature_id,
            %task_id,
            "update_project_task: query failed"
        );
        internal_error()
    })?;

    let Some(row) = row else {
        warn!(
            user_id = %user.id,
            %project_id,
            %feature_id,
            %task_id,
            "api: PATCH task -> 404"
        );
        return Err(not_found("Task not found."));
    };

    info!(
        user_id = %user.id,
        %project_id,
        %feature_id,
        task_id = %row.id,
        task_status = %row.status,
        task_created_by = %row.created_by,
        "api: PATCH /api/v1/projects/:id/features/:feature_id/tasks/:task_id -> 200 OK"
    );

    Ok(Json(row))
}

fn is_allowed_document_name(file_name: &str) -> bool {
    matches!(
        file_extension(file_name).as_str(),
        "md" | "txt"
            | "csv"
            | "xls"
            | "xlsx"
            | "doc"
            | "docx"
            | "jpg"
            | "jpeg"
            | "png"
            | "gif"
            | "webp"
            | "bmp"
            | "svg"
    )
}

fn file_extension(file_name: &str) -> String {
    FsPath::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn safe_document_file_name(original_name: &str) -> String {
    let base = original_name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("document")
        .trim();

    let sanitized: String = base
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();

    let trimmed = sanitized.trim_matches('.');
    if trimmed.is_empty() {
        "document".to_string()
    } else {
        trimmed.to_string()
    }
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
