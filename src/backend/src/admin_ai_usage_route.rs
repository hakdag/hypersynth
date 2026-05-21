use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::ai_usage_query_helpers::{
    internal_error, pagination_limit, pagination_offset, resolve_range, DEFAULT_RANGE_DAYS,
};
use crate::app_state::AppState;
use crate::auth_route::require_system_admin;
use crate::types::{
    AdminAiUsageByCompanyQuery, AdminAiUsageByCompanyRow, AdminAiUsageByProviderModelQuery,
    AdminAiUsageByProviderModelRow, AdminAiUsageByUserQuery, AdminAiUsageByUserRow,
    AdminAiUsageFailureQuery, AdminAiUsageFailureRow, AdminAiUsageHighUsageSort,
    AdminAiUsageRangeQuery, AdminAiUsageSummary, AdminAiUsageTotals, ApiErrorBody,
};

pub async fn admin_ai_usage_summary(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<AdminAiUsageRangeQuery>,
) -> Result<Json<AdminAiUsageTotals>, (StatusCode, Json<ApiErrorBody>)> {
    let _admin_email = require_system_admin(&state.pool, &jar).await?;
    let (from, to) = resolve_range(query.from, query.to)?;
    let totals = fetch_totals(&state.pool, from, to, None)
        .await
        .map_err(|_| internal_error())?;
    Ok(Json(totals))
}

pub async fn admin_ai_usage_by_company(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<AdminAiUsageByCompanyQuery>,
) -> Result<Json<Vec<AdminAiUsageByCompanyRow>>, (StatusCode, Json<ApiErrorBody>)> {
    let _admin_email = require_system_admin(&state.pool, &jar).await?;
    let (from, to) = resolve_range(query.from, query.to)?;
    let limit = pagination_limit(query.limit);
    let offset = pagination_offset(query.offset);

    let order_clause = match query.sort {
        AdminAiUsageHighUsageSort::Cost => {
            "ORDER BY COALESCE(SUM(u.estimated_cost_micros), 0) DESC"
        }
        AdminAiUsageHighUsageSort::Tokens => {
            "ORDER BY (COALESCE(SUM(u.input_tokens), 0) + COALESCE(SUM(u.output_tokens), 0)) DESC"
        }
    };

    let sql = format!(
        r#"
        SELECT
            u.company_id,
            c.name AS company_name,
            COUNT(*)::bigint AS request_count,
            COALESCE(SUM(u.input_tokens), 0)::bigint AS input_tokens,
            COALESCE(SUM(u.output_tokens), 0)::bigint AS output_tokens,
            (COALESCE(SUM(u.input_tokens), 0) + COALESCE(SUM(u.output_tokens), 0))::bigint AS total_tokens,
            (COALESCE(SUM(u.estimated_cost_micros), 0)::double precision / 1000000.0) AS estimated_cost,
            COUNT(*) FILTER (WHERE u.status = 'success')::bigint AS success_count,
            COUNT(*) FILTER (WHERE u.status = 'failed')::bigint AS failure_count
        FROM ai_usage u
        LEFT JOIN companies c ON c.id = u.company_id
        WHERE u.created_at >= $1 AND u.created_at < $2
        GROUP BY u.company_id, c.name
        {order_clause}
        LIMIT $3 OFFSET $4
        "#
    );

    let rows = sqlx::query_as::<_, AdminAiUsageByCompanyRow>(&sql)
        .bind(from)
        .bind(to)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
        .map_err(|_| internal_error())?;

    Ok(Json(rows))
}

pub async fn admin_ai_usage_by_user(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<AdminAiUsageByUserQuery>,
) -> Result<Json<Vec<AdminAiUsageByUserRow>>, (StatusCode, Json<ApiErrorBody>)> {
    let _admin_email = require_system_admin(&state.pool, &jar).await?;
    let (from, to) = resolve_range(query.from, query.to)?;
    let limit = pagination_limit(query.limit);
    let offset = pagination_offset(query.offset);

    let rows = sqlx::query_as::<_, AdminAiUsageByUserRow>(
        r#"
        SELECT
            u.user_id,
            usr.email AS user_email,
            usr.fullname AS user_full_name,
            u.company_id,
            c.name AS company_name,
            COUNT(*)::bigint AS request_count,
            COALESCE(SUM(u.input_tokens), 0)::bigint AS input_tokens,
            COALESCE(SUM(u.output_tokens), 0)::bigint AS output_tokens,
            (COALESCE(SUM(u.input_tokens), 0) + COALESCE(SUM(u.output_tokens), 0))::bigint AS total_tokens,
            (COALESCE(SUM(u.estimated_cost_micros), 0)::double precision / 1000000.0) AS estimated_cost,
            COUNT(*) FILTER (WHERE u.status = 'success')::bigint AS success_count,
            COUNT(*) FILTER (WHERE u.status = 'failed')::bigint AS failure_count
        FROM ai_usage u
        INNER JOIN users usr ON usr.id = u.user_id
        LEFT JOIN companies c ON c.id = u.company_id
        WHERE u.created_at >= $1 AND u.created_at < $2
          AND ($3::uuid IS NULL OR u.company_id = $3)
        GROUP BY u.user_id, usr.email, usr.fullname, u.company_id, c.name
        ORDER BY (COALESCE(SUM(u.input_tokens), 0) + COALESCE(SUM(u.output_tokens), 0)) DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(from)
    .bind(to)
    .bind(query.company_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| internal_error())?;

    Ok(Json(rows))
}

pub async fn admin_ai_usage_by_provider_model(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<AdminAiUsageByProviderModelQuery>,
) -> Result<Json<Vec<AdminAiUsageByProviderModelRow>>, (StatusCode, Json<ApiErrorBody>)> {
    let _admin_email = require_system_admin(&state.pool, &jar).await?;
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
        WHERE u.created_at >= $1 AND u.created_at < $2
          AND ($3::uuid IS NULL OR u.company_id = $3)
        GROUP BY u.provider, u.model
        ORDER BY (COALESCE(SUM(u.input_tokens), 0) + COALESCE(SUM(u.output_tokens), 0)) DESC
        "#,
    )
    .bind(from)
    .bind(to)
    .bind(query.company_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| internal_error())?;

    Ok(Json(rows))
}

pub async fn admin_ai_usage_failures(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<AdminAiUsageFailureQuery>,
) -> Result<Json<Vec<AdminAiUsageFailureRow>>, (StatusCode, Json<ApiErrorBody>)> {
    let _admin_email = require_system_admin(&state.pool, &jar).await?;
    let (from, to) = resolve_range(query.from, query.to)?;
    let limit = pagination_limit(query.limit);
    let offset = pagination_offset(query.offset);

    let provider = query
        .provider
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let rows = sqlx::query_as::<_, AdminAiUsageFailureRow>(
        r#"
        SELECT
            u.id,
            u.company_id,
            c.name AS company_name,
            u.user_id,
            usr.email AS user_email,
            u.provider,
            u.model,
            u.operation_type,
            u.error_code,
            u.created_at
        FROM ai_usage u
        INNER JOIN users usr ON usr.id = u.user_id
        LEFT JOIN companies c ON c.id = u.company_id
        WHERE u.status = 'failed'
          AND u.created_at >= $1 AND u.created_at < $2
          AND ($3::uuid IS NULL OR u.company_id = $3)
          AND ($4::uuid IS NULL OR u.user_id = $4)
          AND ($5::text IS NULL OR u.provider = $5)
        ORDER BY u.created_at DESC
        LIMIT $6 OFFSET $7
        "#,
    )
    .bind(from)
    .bind(to)
    .bind(query.company_id)
    .bind(query.user_id)
    .bind(provider.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| internal_error())?;

    Ok(Json(rows))
}

pub async fn company_ai_usage_summary(
    pool: &PgPool,
    company_id: Uuid,
) -> Result<AdminAiUsageSummary, sqlx::Error> {
    let to = Utc::now();
    let from = to - Duration::days(DEFAULT_RANGE_DAYS);
    let totals = fetch_totals(pool, from, to, Some(company_id)).await?;
    Ok(AdminAiUsageSummary {
        total_requests: totals.request_count,
        total_tokens: totals.total_tokens,
        estimated_cost: totals.estimated_cost,
    })
}

async fn fetch_totals(
    pool: &PgPool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    company_id: Option<Uuid>,
) -> Result<AdminAiUsageTotals, sqlx::Error> {
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
        WHERE created_at >= $1 AND created_at < $2
          AND ($3::uuid IS NULL OR company_id = $3)
        "#,
    )
    .bind(from)
    .bind(to)
    .bind(company_id)
    .fetch_one(pool)
    .await?;

    let (request_count, input_tokens, output_tokens, estimated_cost, success_count, failure_count) =
        row;

    Ok(AdminAiUsageTotals {
        request_count,
        input_tokens,
        output_tokens,
        total_tokens: input_tokens + output_tokens,
        estimated_cost,
        success_count,
        failure_count,
    })
}

