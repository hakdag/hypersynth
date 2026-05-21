use sqlx::PgPool;
use tracing::warn;

use crate::types::{
    AiTokenUsage, AiUsageRecord, AiUsageScope, AiUsageStatus, ProviderId,
};

pub struct AiUsageService;

impl AiUsageService {
    pub async fn record_success(pool: &PgPool, scope: AiUsageScope<'_>, usage: AiTokenUsage) {
        let estimated_cost_micros =
            match Self::lookup_cost_micros(pool, scope.provider, scope.model, usage).await {
                Ok(cost) => cost,
                Err(e) => {
                    warn!(error = %e, "ai_usage: cost lookup failed, defaulting to 0");
                    0
                }
            };

        let record = AiUsageRecord {
            company_id: scope.company_id,
            user_id: scope.user_id,
            project_id: scope.project_id,
            feature_id: scope.feature_id,
            operation_type: scope.operation_type,
            provider: scope.provider,
            model: scope.model.to_string(),
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            estimated_cost_micros,
            status: AiUsageStatus::Success,
            error_code: None,
        };

        if let Err(e) = Self::insert(pool, &record).await {
            warn!(error = %e, "ai_usage: failed to record success");
        }
    }

    pub async fn record_failure(pool: &PgPool, scope: AiUsageScope<'_>, error_code: String) {
        let record = AiUsageRecord {
            company_id: scope.company_id,
            user_id: scope.user_id,
            project_id: scope.project_id,
            feature_id: scope.feature_id,
            operation_type: scope.operation_type,
            provider: scope.provider,
            model: scope.model.to_string(),
            input_tokens: 0,
            output_tokens: 0,
            estimated_cost_micros: 0,
            status: AiUsageStatus::Failed,
            error_code: Some(error_code),
        };

        if let Err(e) = Self::insert(pool, &record).await {
            warn!(error = %e, "ai_usage: failed to record failure");
        }
    }

    async fn insert(pool: &PgPool, record: &AiUsageRecord) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO ai_usage (
                company_id,
                user_id,
                project_id,
                feature_id,
                operation_type,
                provider,
                model,
                input_tokens,
                output_tokens,
                estimated_cost_micros,
                status,
                error_code
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(record.company_id)
        .bind(record.user_id)
        .bind(record.project_id)
        .bind(record.feature_id)
        .bind(record.operation_type.as_str())
        .bind(record.provider.as_str())
        .bind(&record.model)
        .bind(i32::try_from(record.input_tokens).unwrap_or(i32::MAX))
        .bind(i32::try_from(record.output_tokens).unwrap_or(i32::MAX))
        .bind(record.estimated_cost_micros)
        .bind(record.status.as_str())
        .bind(record.error_code.as_deref())
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn lookup_cost_micros(
        pool: &PgPool,
        provider: ProviderId,
        model: &str,
        usage: AiTokenUsage,
    ) -> Result<i64, sqlx::Error> {
        let row: Option<(i64, i64)> = sqlx::query_as(
            r#"
            SELECT input_cost_per_1k_micros, output_cost_per_1k_micros
            FROM ai_model_pricing
            WHERE provider = $1 AND model = $2
            "#,
        )
        .bind(provider.as_str())
        .bind(model)
        .fetch_optional(pool)
        .await?;

        let Some((input_per_1k, output_per_1k)) = row else {
            return Ok(0);
        };

        let input_cost = (i64::from(usage.input_tokens) * input_per_1k) / 1000;
        let output_cost = (i64::from(usage.output_tokens) * output_per_1k) / 1000;
        Ok(input_cost + output_cost)
    }
}
