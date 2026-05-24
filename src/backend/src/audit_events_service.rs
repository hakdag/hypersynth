use serde_json::Value;
use sqlx::PgPool;
use tracing::warn;

use crate::types::{AuditContext, AuditEventType};

pub struct AuditEventsService;

impl AuditEventsService {
    /// Fire-and-forget write into `audit_events` using a fresh pool
    /// connection so the recording survives a rollback of the request
    /// transaction. Failures are logged but never propagated (SF-24:
    /// recording an audit event must not block or break the originating
    /// action).
    pub async fn record_with_pool(
        pool: &PgPool,
        event_type: AuditEventType,
        ctx: &AuditContext,
        payload: Value,
    ) {
        if let Err(e) = Self::insert(pool, event_type, ctx, payload).await {
            warn!(
                event_type = event_type.as_str(),
                error = %e,
                "audit_events: insert failed"
            );
        }
    }

    async fn insert(
        pool: &PgPool,
        event_type: AuditEventType,
        ctx: &AuditContext,
        payload: Value,
    ) -> Result<(), sqlx::Error> {
        let ts_ms = chrono::Utc::now().timestamp_millis();
        let ip_text = ctx.ip_address.map(|ip| ip.to_string());

        sqlx::query(
            r#"
            INSERT INTO audit_events (
                ts_ms, event_type, actor, payload,
                request_id, ip_address, user_agent
            )
            VALUES ($1, $2, $3, $4, $5, $6::inet, $7)
            "#,
        )
        .bind(ts_ms)
        .bind(event_type.as_str())
        .bind(&ctx.actor)
        .bind(&payload)
        .bind(ctx.request_id)
        .bind(ip_text)
        .bind(ctx.user_agent.as_deref())
        .execute(pool)
        .await?;

        Ok(())
    }
}
