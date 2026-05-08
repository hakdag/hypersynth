use sqlx::Postgres;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::crypto::ApiKeyCipher;
use crate::runtime_decrypt_error::RuntimeDecryptError;
use crate::types::ApiKeyAuditEvent;

/// Service responsible for recording AI API key lifecycle events and
/// decrypting stored ciphertext for the server-side AI execution path.
///
/// All methods are associated functions; the type carries no state. The
/// service deliberately never persists, returns, or logs key material.
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

    /// Loads the encrypted API key for a project owned by `user_id`,
    /// decrypts it using the application secret, and records a
    /// `runtime_use` audit event on success.
    ///
    /// Returns `Ok(None)` when the project exists for this owner but has no
    /// configured key, or when the project does not exist for this owner
    /// (callers should not be able to distinguish the two cases through
    /// this service).
    ///
    /// Intentionally unused at the route layer in this sub-feature; SF-17
    /// ships the runtime entry point ahead of the AI execution paths that
    /// will consume it.
    #[allow(dead_code)]
    pub async fn decrypt_for_runtime(
        state: &AppState,
        project_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<String>, RuntimeDecryptError> {
        let row: Option<(Option<Vec<u8>>,)> = sqlx::query_as(
            r#"
            SELECT encrypted_api_key
            FROM projects
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(project_id)
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await?;

        let Some((Some(ciphertext),)) = row else {
            return Ok(None);
        };

        let cipher = ApiKeyCipher::new(&state.api_key_encryption_key);
        let plaintext = cipher.decrypt(&ciphertext)?;

        Self::record_audit(
            &state.pool,
            project_id,
            user_id,
            ApiKeyAuditEvent::RuntimeUse,
        )
        .await?;

        Ok(Some(plaintext))
    }
}
