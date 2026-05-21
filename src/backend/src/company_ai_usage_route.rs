use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use uuid::Uuid;

use crate::ai_usage_query_helpers::{
    internal_error, pagination_limit, pagination_offset, resolve_range,
};
use crate::app_state::AppState;
use crate::auth_route::require_authenticated_user;
use crate::authorization::{forbidden, require_company_role, VIEW_COMPANY_AI_USAGE};
use crate::types::{
    AdminAiUsageByProviderModelRow, AdminAiUsageTotals, ApiErrorBody, CompanyAiUsageByProjectRow,
    CompanyAiUsageByUserRow, CompanyAiUsageFailureQuery, CompanyAiUsageFailureRow,
    CompanyAiUsageListQuery, CompanyAiUsageRangeQuery, SessionUser,
};

async fn require_company_admin_company(
    state: &AppState,
    jar: &CookieJar,
) -> Result<(SessionUser, Uuid), (StatusCode, Json<ApiErrorBody>)> {
    let user = require_authenticated_user(&state.pool, jar).await?;
    require_company_role(&user, VIEW_COMPANY_AI_USAGE).await?;
    let company_id = user
        .company_id
        .ok_or_else(|| forbidden("You do not have permission to perform this action."))?;
    Ok((user, company_id))
}

pub async fn company_ai_usage_summary(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<CompanyAiUsageRangeQuery>,
) -> Result<Json<AdminAiUsageTotals>, (StatusCode, Json<ApiErrorBody>)> {
    let (_user, company_id) = require_company_admin_company(&state, &jar).await?;
    let (from, to) = resolve_range(query.from, query.to)?;

    let row: (i64, i64, i64, f64, i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint,
            COALESCE(SUM(input_tokens), 0)::bigint,
            COALESCE(SUM(output_tokens), 0)::bigint,
            (COALESCE(SUM(estimated_cost_micros), 0)::double precision / 1000000.0),
            COUNT(*) FILTER (WHERE status = 'success')::bigint,
            COUNT(*) FILTER (WHERE status = 'failed')::bigint
        FROM ai_usage
        WHERE company_id = $1
          AND created_at >= $2 AND created_at < $3
        "#,
    )
    .bind(company_id)
    .bind(from)
    .bind(to)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| internal_error())?;

    let (request_count, input_tokens, output_tokens, estimated_cost, success_count, failure_count) =
        row;

    Ok(Json(AdminAiUsageTotals {
        request_count,
        input_tokens,
        output_tokens,
        total_tokens: input_tokens + output_tokens,
        estimated_cost,
        success_count,
        failure_count,
    }))
}

pub async fn company_ai_usage_by_user(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<CompanyAiUsageListQuery>,
) -> Result<Json<Vec<CompanyAiUsageByUserRow>>, (StatusCode, Json<ApiErrorBody>)> {
    let (_user, company_id) = require_company_admin_company(&state, &jar).await?;
    let (from, to) = resolve_range(query.from, query.to)?;
    let limit = pagination_limit(query.limit);
    let offset = pagination_offset(query.offset);

    let rows = sqlx::query_as::<_, CompanyAiUsageByUserRow>(
        r#"
        SELECT
            u.user_id,
            usr.email AS user_email,
            usr.fullname AS user_full_name,
            COUNT(*)::bigint AS request_count,
            COALESCE(SUM(u.input_tokens), 0)::bigint AS input_tokens,
            COALESCE(SUM(u.output_tokens), 0)::bigint AS output_tokens,
            (COALESCE(SUM(u.input_tokens), 0) + COALESCE(SUM(u.output_tokens), 0))::bigint AS total_tokens,
            (COALESCE(SUM(u.estimated_cost_micros), 0)::double precision / 1000000.0) AS estimated_cost,
            COUNT(*) FILTER (WHERE u.status = 'success')::bigint AS success_count,
            COUNT(*) FILTER (WHERE u.status = 'failed')::bigint AS failure_count
        FROM ai_usage u
        INNER JOIN users usr ON usr.id = u.user_id
        WHERE u.company_id = $1
          AND u.created_at >= $2 AND u.created_at < $3
        GROUP BY u.user_id, usr.email, usr.fullname
        ORDER BY (COALESCE(SUM(u.input_tokens), 0) + COALESCE(SUM(u.output_tokens), 0)) DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(company_id)
    .bind(from)
    .bind(to)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| internal_error())?;

    Ok(Json(rows))
}

pub async fn company_ai_usage_by_project(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<CompanyAiUsageListQuery>,
) -> Result<Json<Vec<CompanyAiUsageByProjectRow>>, (StatusCode, Json<ApiErrorBody>)> {
    let (_user, company_id) = require_company_admin_company(&state, &jar).await?;
    let (from, to) = resolve_range(query.from, query.to)?;
    let limit = pagination_limit(query.limit);
    let offset = pagination_offset(query.offset);

    let rows = sqlx::query_as::<_, CompanyAiUsageByProjectRow>(
        r#"
        SELECT
            u.project_id,
            p.name AS project_name,
            COUNT(*)::bigint AS request_count,
            COALESCE(SUM(u.input_tokens), 0)::bigint AS input_tokens,
            COALESCE(SUM(u.output_tokens), 0)::bigint AS output_tokens,
            (COALESCE(SUM(u.input_tokens), 0) + COALESCE(SUM(u.output_tokens), 0))::bigint AS total_tokens,
            (COALESCE(SUM(u.estimated_cost_micros), 0)::double precision / 1000000.0) AS estimated_cost,
            COUNT(*) FILTER (WHERE u.status = 'success')::bigint AS success_count,
            COUNT(*) FILTER (WHERE u.status = 'failed')::bigint AS failure_count
        FROM ai_usage u
        LEFT JOIN projects p ON p.id = u.project_id AND p.company_id = $1
        WHERE u.company_id = $1
          AND u.created_at >= $2 AND u.created_at < $3
        GROUP BY u.project_id, p.name
        ORDER BY (COALESCE(SUM(u.input_tokens), 0) + COALESCE(SUM(u.output_tokens), 0)) DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(company_id)
    .bind(from)
    .bind(to)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| internal_error())?;

    Ok(Json(rows))
}

pub async fn company_ai_usage_by_provider_model(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<CompanyAiUsageRangeQuery>,
) -> Result<Json<Vec<AdminAiUsageByProviderModelRow>>, (StatusCode, Json<ApiErrorBody>)> {
    let (_user, company_id) = require_company_admin_company(&state, &jar).await?;
    let (from, to) = resolve_range(query.from, query.to)?;

    let rows = sqlx::query_as::<_, AdminAiUsageByProviderModelRow>(
        r#"
        SELECT
            u.provider,
            u.model,
            COUNT(*)::bigint AS request_count,
            COALESCE(SUM(u.input_tokens), 0)::bigint AS input_tokens,
            COALESCE(SUM(u.output_tokens), 0)::bigint AS output_tokens,
            (COALESCE(SUM(u.input_tokens), 0) + COALESCE(SUM(u.output_tokens), 0))::bigint AS total_tokens,
            (COALESCE(SUM(u.estimated_cost_micros), 0)::double precision / 1000000.0) AS estimated_cost,
            COUNT(*) FILTER (WHERE u.status = 'success')::bigint AS success_count,
            COUNT(*) FILTER (WHERE u.status = 'failed')::bigint AS failure_count
        FROM ai_usage u
        WHERE u.company_id = $1
          AND u.created_at >= $2 AND u.created_at < $3
        GROUP BY u.provider, u.model
        ORDER BY (COALESCE(SUM(u.input_tokens), 0) + COALESCE(SUM(u.output_tokens), 0)) DESC
        "#,
    )
    .bind(company_id)
    .bind(from)
    .bind(to)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| internal_error())?;

    Ok(Json(rows))
}

pub async fn company_ai_usage_failures(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<CompanyAiUsageFailureQuery>,
) -> Result<Json<Vec<CompanyAiUsageFailureRow>>, (StatusCode, Json<ApiErrorBody>)> {
    let (_user, company_id) = require_company_admin_company(&state, &jar).await?;
    let (from, to) = resolve_range(query.from, query.to)?;
    let limit = pagination_limit(query.limit);
    let offset = pagination_offset(query.offset);

    let provider = query
        .provider
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let rows = sqlx::query_as::<_, CompanyAiUsageFailureRow>(
        r#"
        SELECT
            u.id,
            u.user_id,
            usr.email AS user_email,
            u.provider,
            u.model,
            u.operation_type,
            u.error_code,
            u.created_at
        FROM ai_usage u
        INNER JOIN users usr ON usr.id = u.user_id
        WHERE u.status = 'failed'
          AND u.company_id = $1
          AND u.created_at >= $2 AND u.created_at < $3
          AND ($4::uuid IS NULL OR u.user_id = $4)
          AND ($5::text IS NULL OR u.provider = $5)
        ORDER BY u.created_at DESC
        LIMIT $6 OFFSET $7
        "#,
    )
    .bind(company_id)
    .bind(from)
    .bind(to)
    .bind(query.user_id)
    .bind(provider.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| internal_error())?;

    Ok(Json(rows))
}
