use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use chrono::{DateTime, Utc};
use tracing::warn;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth_route;
use crate::authorization;
use crate::tenant_scope_service::TenantScopeService;
use crate::types::{
    AccountType, AddProjectMemberRequest, ApiErrorBody, CompanyRole, ProjectMemberResponse,
    ProjectMembershipRole, TenantScope,
};

async fn project_in_company_scope(
    pool: &sqlx::PgPool,
    project_id: Uuid,
    scope: TenantScope,
) -> Result<bool, (StatusCode, Json<ApiErrorBody>)> {
    let (company_id, owner_id, is_admin, user_id) = scope.project_access_binds();
    let found: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT id
        FROM projects
        WHERE id = $1
          AND (
            ($3::uuid IS NOT NULL AND owner_user_id = $3 AND company_id IS NULL)
            OR
            ($2::uuid IS NOT NULL AND company_id = $2 AND (
                $4::boolean
                OR EXISTS (
                    SELECT 1 FROM project_memberships pm
                    WHERE pm.project_id = projects.id AND pm.user_id = $5
                )
            ))
          )
        "#,
    )
    .bind(project_id)
    .bind(company_id)
    .bind(owner_id)
    .bind(is_admin)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| internal_error())?;

    Ok(found.is_some())
}

async fn caller_is_project_manager_member(
    pool: &sqlx::PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<bool, (StatusCode, Json<ApiErrorBody>)> {
    let ok: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM project_memberships pm
            WHERE pm.project_id = $1
              AND pm.user_id = $2
              AND pm.role = 'project_manager'
        )
        "#,
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|_| internal_error())?;
    Ok(ok)
}

async fn ensure_can_list_members(
    pool: &sqlx::PgPool,
    scope: TenantScope,
    project_id: Uuid,
) -> Result<(), (StatusCode, Json<ApiErrorBody>)> {
    if !project_in_company_scope(pool, project_id, scope).await? {
        return Err(not_found_project());
    }
    if scope.is_company_admin() {
        return Ok(());
    }
    let uid = scope.session_user_id();
    let is_member: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM project_memberships pm
            WHERE pm.project_id = $1 AND pm.user_id = $2
        )
        "#,
    )
    .bind(project_id)
    .bind(uid)
    .fetch_one(pool)
    .await
    .map_err(|_| internal_error())?;
    if is_member {
        Ok(())
    } else {
        Err(not_found_project())
    }
}

async fn ensure_can_manage_members(
    pool: &sqlx::PgPool,
    scope: TenantScope,
    project_id: Uuid,
) -> Result<(), (StatusCode, Json<ApiErrorBody>)> {
    if !project_in_company_scope(pool, project_id, scope).await? {
        return Err(not_found_project());
    }
    if scope.is_company_admin() {
        return Ok(());
    }
    let uid = scope.session_user_id();
    if caller_is_project_manager_member(pool, project_id, uid).await? {
        Ok(())
    } else {
        Err(authorization::forbidden(
            "You do not have permission to perform this action.",
        ))
    }
}

pub async fn list_project_members(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<ProjectMemberResponse>>, (StatusCode, Json<ApiErrorBody>)> {
    let user = auth_route::require_authenticated_user(&state.pool, &jar).await?;
    if user.account_type != AccountType::Company {
        return Err(authorization::forbidden(
            "You do not have permission to perform this action.",
        ));
    }
    let scope = TenantScopeService::from_session(&user)?;
    let TenantScope::Company { company_id, .. } = scope else {
        return Err(authorization::forbidden(
            "You do not have permission to perform this action.",
        ));
    };

    ensure_can_list_members(&state.pool, scope, project_id).await?;

    let rows = sqlx::query_as::<_, (Uuid, String, String, Option<String>, String, DateTime<Utc>)>(
        r#"
        SELECT u.id, u.fullname, u.email, u.role, pm.role, pm.created_at
        FROM project_memberships pm
        INNER JOIN users u ON u.id = pm.user_id
        INNER JOIN projects p ON p.id = pm.project_id
        WHERE pm.project_id = $1
          AND p.company_id = $2
        ORDER BY pm.created_at ASC
        "#,
    )
    .bind(project_id)
    .bind(company_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        warn!(error = %e, "list_project_members: query failed");
        internal_error()
    })?;

    let mut out = Vec::with_capacity(rows.len());
    for (id, fullname, email, role_raw, project_role_raw, created_at) in rows {
        let company_role = role_raw
            .as_deref()
            .and_then(CompanyRole::from_db_value);
        let project_role = ProjectMembershipRole::from_db_value(project_role_raw.as_str())
            .ok_or_else(internal_error)?;
        out.push(ProjectMemberResponse {
            user_id: id,
            fullname,
            email,
            company_role,
            project_role,
            created_at,
        });
    }

    Ok(Json(out))
}

pub async fn add_project_member(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<AddProjectMemberRequest>,
) -> Result<Json<ProjectMemberResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let user = auth_route::require_authenticated_user(&state.pool, &jar).await?;
    if user.account_type != AccountType::Company {
        return Err(authorization::forbidden(
            "You do not have permission to perform this action.",
        ));
    }
    let scope = TenantScopeService::from_session(&user)?;
    let TenantScope::Company { company_id, .. } = scope else {
        return Err(authorization::forbidden(
            "You do not have permission to perform this action.",
        ));
    };

    ensure_can_manage_members(&state.pool, scope, project_id).await?;

    let in_company: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM company_users cu
            WHERE cu.company_id = $1 AND cu.user_id = $2
        )
        "#,
    )
    .bind(company_id)
    .bind(payload.user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| internal_error())?;

    if !in_company {
        return Err(not_found_project());
    }

    sqlx::query(
        r#"
        INSERT INTO project_memberships (project_id, user_id, role)
        VALUES ($1, $2, $3)
        ON CONFLICT (project_id, user_id) DO NOTHING
        "#,
    )
    .bind(project_id)
    .bind(payload.user_id)
    .bind(payload.project_role.as_db_value())
    .execute(&state.pool)
    .await
    .map_err(|_| internal_error())?;

    let row = sqlx::query_as::<_, (Uuid, String, String, Option<String>, String, DateTime<Utc>)>(
        r#"
        SELECT u.id, u.fullname, u.email, u.role, pm.role, pm.created_at
        FROM project_memberships pm
        INNER JOIN users u ON u.id = pm.user_id
        INNER JOIN projects p ON p.id = pm.project_id
        WHERE pm.project_id = $1
          AND pm.user_id = $2
          AND p.company_id = $3
        "#,
    )
    .bind(project_id)
    .bind(payload.user_id)
    .bind(company_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| internal_error())?
    .ok_or_else(not_found_project)?;

    let (id, fullname, email, role_raw, project_role_raw, created_at) = row;
    let company_role = role_raw
        .as_deref()
        .and_then(CompanyRole::from_db_value);
    let project_role = ProjectMembershipRole::from_db_value(project_role_raw.as_str())
        .ok_or_else(internal_error)?;

    Ok(Json(ProjectMemberResponse {
        user_id: id,
        fullname,
        email,
        company_role,
        project_role,
        created_at,
    }))
}

pub async fn remove_project_member(
    State(state): State<AppState>,
    jar: CookieJar,
    Path((project_id, member_user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<ApiErrorBody>)> {
    let user = auth_route::require_authenticated_user(&state.pool, &jar).await?;
    if user.account_type != AccountType::Company {
        return Err(authorization::forbidden(
            "You do not have permission to perform this action.",
        ));
    }
    let scope = TenantScopeService::from_session(&user)?;
    let TenantScope::Company { company_id, .. } = scope else {
        return Err(authorization::forbidden(
            "You do not have permission to perform this action.",
        ));
    };

    ensure_can_manage_members(&state.pool, scope, project_id).await?;

    sqlx::query(
        r#"
        DELETE FROM project_memberships pm
        USING projects p
        WHERE pm.project_id = p.id
          AND pm.project_id = $1
          AND pm.user_id = $2
          AND p.company_id = $3
        "#,
    )
    .bind(project_id)
    .bind(member_user_id)
    .bind(company_id)
    .execute(&state.pool)
    .await
    .map_err(|_| internal_error())?;

    Ok(StatusCode::NO_CONTENT)
}

fn not_found_project() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiErrorBody {
            message: "Project not found.".into(),
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
