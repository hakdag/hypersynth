use sqlx::Postgres;
use uuid::Uuid;

use crate::types::ApiKeyAuditEvent;

/// Service responsible for recording AI API key lifecycle events.
pub struct ProjectApiKeyService;

impl ProjectApiKeyService {
    /// Inserts an audit row describing a key lifecycle event. The executor
    /// can be either a pool or a transaction so callers can keep the audit
    /// write atomic with the data change that produced it.
    pub async fn record_audit<'e, E>(
        executor: E,
        project_id: Uuid,
        user_id: Uuid,
        event: ApiKeyAuditEvent,
    ) -> Result<(), sqlx::Error>
    where
        E: sqlx::Executor<'e, Database = Postgres>,
    {
        sqlx::query(
            r#"
            INSERT INTO project_api_key_audit (project_id, user_id, event_type)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(project_id)
        .bind(user_id)
        .bind(event.as_str())
        .execute(executor)
        .await?;
        Ok(())
    }
}
