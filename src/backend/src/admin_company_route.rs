use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use sqlx::PgPool;
use uuid::Uuid;

use crate::admin_ai_usage_route::company_ai_usage_summary;
use crate::app_state::AppState;
use crate::auth_route::require_system_admin;
use crate::types::{
    AdminCompaniesListQuery, AdminCompanyDetail, AdminCompanySummary, ApiErrorBody, CompanyStatus,
    UpdateCompanyStatusRequest,
};

const DEFAULT_LIST_LIMIT: i64 = 50;
const MAX_LIST_LIMIT: i64 = 200;

pub async fn list_admin_companies(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<AdminCompaniesListQuery>,
) -> Result<Json<Vec<AdminCompanySummary>>, (StatusCode, Json<ApiErrorBody>)> {
    let _admin_email = require_system_admin(&state.pool, &jar).await?;

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

    let rows = sqlx::query_as::<_, AdminCompanySummary>(
        r#"
        SELECT
            c.id,
            c.name,
            c.company_email,
            c.status,
            c.created_at,
            COALESCE(uc.cnt, 0) AS user_count,
            COALESCE(pc.cnt, 0) AS project_count,
            COALESCE(dc.cnt, 0) AS document_count
        FROM companies c
        LEFT JOIN (
            SELECT company_id, COUNT(*)::bigint AS cnt
            FROM company_users
            GROUP BY company_id
        ) uc ON uc.company_id = c.id
        LEFT JOIN (
            SELECT company_id, COUNT(*)::bigint AS cnt
            FROM projects
            WHERE company_id IS NOT NULL
            GROUP BY company_id
        ) pc ON pc.company_id = c.id
        LEFT JOIN (
            SELECT p.company_id, COUNT(*)::bigint AS cnt
            FROM project_documents d
            INNER JOIN projects p ON p.id = d.project_id
            WHERE p.company_id IS NOT NULL
            GROUP BY p.company_id
        ) dc ON dc.company_id = c.id
        WHERE (
            $1::text IS NULL
            OR c.name ILIKE '%' || $1 || '%'
            OR c.company_email ILIKE '%' || $1 || '%'
        )
        ORDER BY lower(c.name) ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(search.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| internal_error())?;

    Ok(Json(rows))
}

pub async fn get_admin_company(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(company_id): Path<Uuid>,
) -> Result<Json<AdminCompanyDetail>, (StatusCode, Json<ApiErrorBody>)> {
    let _admin_email = require_system_admin(&state.pool, &jar).await?;

    let mut detail = fetch_admin_company_detail(&state.pool, company_id)
        .await?
        .ok_or_else(not_found)?;
    attach_company_ai_usage(&state.pool, &mut detail).await;

    Ok(Json(detail))
}

pub async fn set_admin_company_status(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(company_id): Path<Uuid>,
    Json(payload): Json<UpdateCompanyStatusRequest>,
) -> Result<Json<AdminCompanyDetail>, (StatusCode, Json<ApiErrorBody>)> {
    let admin_email = require_system_admin(&state.pool, &jar).await?;

    match payload.status {
        CompanyStatus::Active | CompanyStatus::Disabled => {}
        CompanyStatus::PendingVerification => {
            return Err(bad_request(
                "Only active or disabled status can be set by a system administrator.",
            ));
        }
    }

    let mut tx = state.pool.begin().await.map_err(|_| internal_error())?;

    let previous_status: Option<String> =
        sqlx::query_scalar("SELECT status FROM companies WHERE id = $1 FOR UPDATE")
            .bind(company_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| internal_error())?;

    let Some(previous_status) = previous_status else {
        return Err(not_found());
    };

    sqlx::query(
        r#"
        UPDATE companies
        SET status = $2, updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(company_id)
    .bind(payload.status.as_db_value())
    .execute(&mut *tx)
    .await
    .map_err(|_| internal_error())?;

    tx.commit().await.map_err(|_| internal_error())?;

    tracing::warn!(
        target: "system_admin_audit",
        admin_email = %admin_email,
        company_id = %company_id,
        previous_status = %previous_status,
        new_status = %payload.status.as_db_value(),
        "company status changed by system admin"
    );

    let mut detail = fetch_admin_company_detail(&state.pool, company_id)
        .await?
        .ok_or_else(not_found)?;
    attach_company_ai_usage(&state.pool, &mut detail).await;

    Ok(Json(detail))
}

async fn attach_company_ai_usage(pool: &PgPool, detail: &mut AdminCompanyDetail) {
    match company_ai_usage_summary(pool, detail.id).await {
        Ok(summary) => detail.ai_usage = Some(summary),
        Err(e) => {
            tracing::warn!(error = %e, company_id = %detail.id, "failed to load company ai usage summary");
            detail.ai_usage = None;
        }
    }
}

async fn fetch_admin_company_detail(
    pool: &PgPool,
    company_id: Uuid,
) -> Result<Option<AdminCompanyDetail>, (StatusCode, Json<ApiErrorBody>)> {
    sqlx::query_as::<_, AdminCompanyDetail>(
        r#"
        SELECT
            c.id,
            c.name,
            c.company_email,
            c.country,
            c.timezone,
            c.legal_name,
            c.website,
            c.industry,
            c.company_size,
            c.phone,
            c.billing_email,
            c.address,
            c.tax_vat_number,
            c.status,
            c.created_at,
            c.updated_at,
            COALESCE(uc.cnt, 0) AS user_count,
            COALESCE(pc.cnt, 0) AS project_count,
            COALESCE(dc.cnt, 0) AS document_count
        FROM companies c
        LEFT JOIN (
            SELECT company_id, COUNT(*)::bigint AS cnt
            FROM company_users
            GROUP BY company_id
        ) uc ON uc.company_id = c.id
        LEFT JOIN (
            SELECT company_id, COUNT(*)::bigint AS cnt
            FROM projects
            WHERE company_id IS NOT NULL
            GROUP BY company_id
        ) pc ON pc.company_id = c.id
        LEFT JOIN (
            SELECT p.company_id, COUNT(*)::bigint AS cnt
            FROM project_documents d
            INNER JOIN projects p ON p.id = d.project_id
            WHERE p.company_id IS NOT NULL
            GROUP BY p.company_id
        ) dc ON dc.company_id = c.id
        WHERE c.id = $1
        "#,
    )
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| internal_error())
}

fn not_found() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiErrorBody::msg("Company not found.")),
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
