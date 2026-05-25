use axum::http::StatusCode;
use axum::Json;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::crypto::ApiKeyCipher;
use crate::platform_config_service::PlatformConfigService;
use crate::project_api_key_service::ProjectApiKeyService;
use crate::runtime_decrypt_error::RuntimeDecryptError;
use crate::types::{
    ApiErrorBody, ApiKeyAuditEvent, CompanyRole, ProjectAiRuntimeSettings,
    ProjectAiSettingsResponse, ProviderId, TenantScope,
};

pub struct ProjectAiSettingsService;

impl ProjectAiSettingsService {
    pub async fn can_manage(
        conn: &mut PgConnection,
        project_id: Uuid,
        scope: TenantScope,
    ) -> Result<bool, sqlx::Error> {
        let company_can_manage = matches!(
            scope,
            TenantScope::Company {
                role: CompanyRole::CompanyAdmin | CompanyRole::ProjectManager,
                ..
            }
        );

        let allowed: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM projects p
                WHERE p.id = $1
                  AND (
                    ($3::uuid IS NOT NULL AND p.owner_user_id = $3 AND p.company_id IS NULL)
                    OR
                    ($2::uuid IS NOT NULL AND p.company_id = $2 AND $4::boolean)
                  )
            )
            "#,
        )
        .bind(project_id)
        .bind(scope.company_id_or_null())
        .bind(scope.owner_user_id_or_null())
        .bind(company_can_manage)
        .fetch_one(&mut *conn)
        .await?;

        Ok(allowed)
    }

    pub async fn authorize_manage(
        conn: &mut PgConnection,
        project_id: Uuid,
        scope: TenantScope,
    ) -> Result<(), (StatusCode, Json<ApiErrorBody>)> {
        let allowed = Self::can_manage(conn, project_id, scope)
            .await
            .map_err(|_| internal_error())?;

        if allowed {
            Ok(())
        } else {
            Err(not_found("Project not found."))
        }
    }

    pub async fn load_response(
        state: &AppState,
        conn: &mut PgConnection,
        project_id: Uuid,
    ) -> Result<ProjectAiSettingsResponse, RuntimeDecryptError> {
        let row: Option<(String, Vec<String>, Option<i64>, bool, Vec<u8>)> = sqlx::query_as(
            r#"
            SELECT provider, allowed_models, monthly_token_limit, usage_tracking_enabled, encrypted_api_key
            FROM project_ai_settings
            WHERE project_id = $1
            "#,
        )
        .bind(project_id)
        .fetch_optional(&mut *conn)
        .await?;

        let Some((
            provider_raw,
            allowed_models,
            monthly_token_limit,
            usage_tracking_enabled,
            encrypted_api_key,
        )) = row
        else {
            return Ok(ProjectAiSettingsResponse {
                project_id,
                provider: None,
                allowed_models: Vec::new(),
                monthly_token_limit: None,
                usage_tracking_enabled: true,
                has_api_key: false,
                api_key_hint: None,
            });
        };

        let provider = provider_raw.parse::<ProviderId>().map_err(|_| {
            RuntimeDecryptError::Database(sqlx::Error::Protocol(
                "unsupported provider stored in project_ai_settings".into(),
            ))
        })?;
        let cipher = ApiKeyCipher::new(&state.api_key_encryption_key);
        let plaintext = cipher.decrypt(&encrypted_api_key)?;

        Ok(ProjectAiSettingsResponse {
            project_id,
            provider: Some(provider),
            allowed_models,
            monthly_token_limit,
            usage_tracking_enabled,
            has_api_key: true,
            api_key_hint: Some(mask_api_key(&plaintext)),
        })
    }

    pub async fn decrypt_existing_api_key(
        state: &AppState,
        conn: &mut PgConnection,
        project_id: Uuid,
    ) -> Result<Option<String>, RuntimeDecryptError> {
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            r#"
            SELECT encrypted_api_key
            FROM project_ai_settings
            WHERE project_id = $1
            "#,
        )
        .bind(project_id)
        .fetch_optional(&mut *conn)
        .await?;

        let Some((encrypted_api_key,)) = row else {
            return Ok(None);
        };

        let cipher = ApiKeyCipher::new(&state.api_key_encryption_key);
        Ok(Some(cipher.decrypt(&encrypted_api_key)?))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert(
        state: &AppState,
        conn: &mut PgConnection,
        project_id: Uuid,
        user_id: Uuid,
        provider: ProviderId,
        allowed_models: Vec<String>,
        monthly_token_limit: Option<i64>,
        usage_tracking_enabled: bool,
        api_key: Option<String>,
        clear_api_key: bool,
    ) -> Result<(), (StatusCode, Json<ApiErrorBody>)> {
        let existing: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT id
            FROM project_ai_settings
            WHERE project_id = $1
            "#,
        )
        .bind(project_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|_| internal_error())?;
        let had_existing = existing.is_some();

        let monthly_token_limit = if monthly_token_limit.is_some() || had_existing {
            monthly_token_limit
        } else {
            PlatformConfigService::load(conn)
                .await
                .map_err(|_| internal_error())?
                .default_monthly_token_limit
        };

        if clear_api_key {
            if had_existing {
                sqlx::query(
                    r#"
                    DELETE FROM project_ai_settings
                    WHERE project_id = $1
                    "#,
                )
                .bind(project_id)
                .execute(&mut *conn)
                .await
                .map_err(|_| internal_error())?;

                ProjectApiKeyService::record_audit(
                    &mut *conn,
                    project_id,
                    user_id,
                    ApiKeyAuditEvent::Cleared,
                )
                .await
                .map_err(|_| internal_error())?;
            }

            return Ok(());
        }

        let encrypted_api_key = match api_key.as_deref() {
            Some(plaintext) => {
                let cipher = ApiKeyCipher::new(&state.api_key_encryption_key);
                Some(cipher.encrypt(plaintext).map_err(|_| internal_error())?)
            }
            None => None,
        };

        if had_existing && encrypted_api_key.is_none() {
            sqlx::query(
                r#"
                UPDATE project_ai_settings
                SET
                    provider = $2,
                    allowed_models = $3,
                    monthly_token_limit = $4,
                    usage_tracking_enabled = $5,
                    updated_at = now()
                WHERE project_id = $1
                "#,
            )
            .bind(project_id)
            .bind(provider.as_str())
            .bind(&allowed_models)
            .bind(monthly_token_limit)
            .bind(usage_tracking_enabled)
            .execute(&mut *conn)
            .await
            .map_err(|_| internal_error())?;
        } else {
            sqlx::query(
                r#"
                INSERT INTO project_ai_settings (
                    project_id,
                    provider,
                    encrypted_api_key,
                    allowed_models,
                    monthly_token_limit,
                    usage_tracking_enabled
                )
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (project_id) DO UPDATE
                SET
                    provider = EXCLUDED.provider,
                    encrypted_api_key = EXCLUDED.encrypted_api_key,
                    allowed_models = EXCLUDED.allowed_models,
                    monthly_token_limit = EXCLUDED.monthly_token_limit,
                    usage_tracking_enabled = EXCLUDED.usage_tracking_enabled,
                    updated_at = now()
                "#,
            )
            .bind(project_id)
            .bind(provider.as_str())
            .bind(encrypted_api_key.as_deref())
            .bind(&allowed_models)
            .bind(monthly_token_limit)
            .bind(usage_tracking_enabled)
            .execute(&mut *conn)
            .await
            .map_err(|_| internal_error())?;
        }

        if encrypted_api_key.is_some() {
            let event = if had_existing {
                ApiKeyAuditEvent::Replaced
            } else {
                ApiKeyAuditEvent::Created
            };
            ProjectApiKeyService::record_audit(&mut *conn, project_id, user_id, event)
                .await
                .map_err(|_| internal_error())?;
        }

        Ok(())
    }

    pub async fn load_for_runtime(
        state: &AppState,
        conn: &mut PgConnection,
        project_id: Uuid,
        scope: TenantScope,
    ) -> Result<Option<ProjectAiRuntimeSettings>, RuntimeDecryptError> {
        let row: Option<(String, Vec<String>, bool, Vec<u8>)> = sqlx::query_as(
            r#"
            SELECT s.provider, s.allowed_models, s.usage_tracking_enabled, s.encrypted_api_key
            FROM project_ai_settings s
            INNER JOIN projects p ON p.id = s.project_id
            WHERE p.id = $1
              AND (
                ($3::uuid IS NOT NULL AND p.owner_user_id = $3 AND p.company_id IS NULL)
                OR
                ($2::uuid IS NOT NULL AND p.company_id = $2 AND (
                    $4::boolean
                    OR EXISTS (
                        SELECT 1 FROM project_memberships pm
                        WHERE pm.project_id = p.id AND pm.user_id = $5
                    )
                ))
              )
            "#,
        )
        .bind(project_id)
        .bind(scope.company_id_or_null())
        .bind(scope.owner_user_id_or_null())
        .bind(scope.is_company_admin())
        .bind(scope.session_user_id())
        .fetch_optional(&mut *conn)
        .await?;

        let Some((provider_raw, allowed_models, usage_tracking_enabled, encrypted_api_key)) = row
        else {
            return Ok(None);
        };

        let provider = provider_raw.parse::<ProviderId>().map_err(|_| {
            RuntimeDecryptError::Database(sqlx::Error::Protocol(
                "unsupported provider stored in project_ai_settings".into(),
            ))
        })?;
        let Some(selected_model) = allowed_models.into_iter().next() else {
            return Ok(None);
        };

        let cipher = ApiKeyCipher::new(&state.api_key_encryption_key);
        let api_key = cipher.decrypt(&encrypted_api_key)?;

        ProjectApiKeyService::record_audit(
            &mut *conn,
            project_id,
            scope.session_user_id(),
            ApiKeyAuditEvent::RuntimeUse,
        )
        .await?;

        Ok(Some(ProjectAiRuntimeSettings {
            provider,
            api_key,
            selected_model,
            usage_tracking_enabled,
        }))
    }
}

fn mask_api_key(plaintext: &str) -> String {
    let mut chars: Vec<char> = plaintext.chars().collect();
    let suffix_start = chars.len().saturating_sub(4);
    let prefix: String = chars.iter().take(2).collect();
    let suffix: String = chars.drain(suffix_start..).collect();

    if prefix.is_empty() {
        format!("****{suffix}")
    } else {
        format!("{prefix}-****{suffix}")
    }
}

fn not_found(message: impl Into<String>) -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiErrorBody {
            message: message.into(),
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
