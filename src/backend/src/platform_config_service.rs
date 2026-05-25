use std::collections::HashMap;

use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgConnection;

use crate::ai::AiProviderRegistry;
use crate::types::{
    ApiErrorBody, PlatformConfig, ProviderId, UpdatePlatformConfigRequest,
};

const PLATFORM_CONFIG_ID: i16 = 1;
const MAX_ANNOUNCEMENT_LEN: usize = 2000;
const AI_REQUESTS_ENABLED_KEY: &str = "ai_requests_enabled";

pub struct PlatformConfigService;

impl PlatformConfigService {
    pub async fn load(conn: &mut PgConnection) -> Result<PlatformConfig, sqlx::Error> {
        let row: Option<PlatformConfigRow> = sqlx::query_as(
            r#"
            SELECT
                allowed_ai_providers,
                default_monthly_token_limit,
                platform_announcement,
                feature_flags,
                updated_at
            FROM platform_config
            WHERE id = $1
            "#,
        )
        .bind(PLATFORM_CONFIG_ID)
        .fetch_optional(&mut *conn)
        .await?;

        Ok(row
            .map(PlatformConfig::from_row)
            .unwrap_or_else(default_config))
    }

    pub async fn update(
        conn: &mut PgConnection,
        patch: UpdatePlatformConfigRequest,
    ) -> Result<PlatformConfig, (StatusCode, Json<ApiErrorBody>)> {
        let current = Self::load(conn)
            .await
            .map_err(|_| internal_error())?;

        let allowed_ai_providers = match patch.allowed_ai_providers {
            Some(providers) => validate_allowed_providers(&providers)?,
            None => current.allowed_ai_providers,
        };

        let default_monthly_token_limit = match patch.default_monthly_token_limit {
            Some(limit) => validate_default_limit(limit)?,
            None => current.default_monthly_token_limit,
        };

        let platform_announcement = match patch.platform_announcement {
            Some(announcement) => validate_announcement(announcement)?,
            None => current.platform_announcement,
        };

        let feature_flags = match patch.feature_flags {
            Some(flags) => validate_feature_flags(flags)?,
            None => current.feature_flags,
        };

        let allowed_for_db: Vec<String> = allowed_ai_providers;

        sqlx::query(
            r#"
            INSERT INTO platform_config (
                id,
                allowed_ai_providers,
                default_monthly_token_limit,
                platform_announcement,
                feature_flags,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (id) DO UPDATE SET
                allowed_ai_providers = EXCLUDED.allowed_ai_providers,
                default_monthly_token_limit = EXCLUDED.default_monthly_token_limit,
                platform_announcement = EXCLUDED.platform_announcement,
                feature_flags = EXCLUDED.feature_flags,
                updated_at = now()
            "#,
        )
        .bind(PLATFORM_CONFIG_ID)
        .bind(&allowed_for_db)
        .bind(default_monthly_token_limit)
        .bind(platform_announcement.as_deref())
        .bind(JsonValue::Object(
            feature_flags
                .iter()
                .map(|(k, v)| (k.clone(), JsonValue::Bool(*v)))
                .collect(),
        ))
        .execute(&mut *conn)
        .await
        .map_err(|_| internal_error())?;

        Self::load(conn).await.map_err(|_| internal_error())
    }

    pub fn allowed_providers(
        config: &PlatformConfig,
        registry: &AiProviderRegistry,
    ) -> Vec<ProviderId> {
        registry
            .supported()
            .into_iter()
            .filter(|id| config.allowed_ai_providers.iter().any(|s| s == id.as_str()))
            .collect()
    }

    pub fn is_ai_enabled(config: &PlatformConfig) -> bool {
        config
            .feature_flags
            .get(AI_REQUESTS_ENABLED_KEY)
            .copied()
            .unwrap_or(true)
    }

    pub fn is_provider_allowed(config: &PlatformConfig, provider: ProviderId) -> bool {
        config
            .allowed_ai_providers
            .iter()
            .any(|s| s == provider.as_str())
    }

    pub async fn require_ai_requests_enabled(
        conn: &mut PgConnection,
    ) -> Result<(), (StatusCode, Json<ApiErrorBody>)> {
        let config = Self::load(conn).await.map_err(|_| internal_error())?;
        if Self::is_ai_enabled(&config) {
            Ok(())
        } else {
            Err((
                StatusCode::FORBIDDEN,
                Json(ApiErrorBody {
                    message: "AI requests are temporarily disabled by the platform administrator."
                        .into(),
                    ..Default::default()
                }),
            ))
        }
    }
}

#[derive(sqlx::FromRow)]
struct PlatformConfigRow {
    allowed_ai_providers: Vec<String>,
    default_monthly_token_limit: Option<i64>,
    platform_announcement: Option<String>,
    feature_flags: JsonValue,
    updated_at: DateTime<Utc>,
}

impl PlatformConfig {
    fn from_row(row: PlatformConfigRow) -> Self {
        Self {
            allowed_ai_providers: row.allowed_ai_providers,
            default_monthly_token_limit: row.default_monthly_token_limit,
            platform_announcement: row.platform_announcement,
            feature_flags: parse_feature_flags(&row.feature_flags),
            updated_at: row.updated_at,
        }
    }
}

fn default_config() -> PlatformConfig {
    PlatformConfig {
        allowed_ai_providers: vec![
            ProviderId::Anthropic.as_str().to_string(),
            ProviderId::OpenAi.as_str().to_string(),
        ],
        default_monthly_token_limit: None,
        platform_announcement: None,
        feature_flags: HashMap::new(),
        updated_at: Utc::now(),
    }
}

fn parse_feature_flags(value: &JsonValue) -> HashMap<String, bool> {
    let mut map = HashMap::new();
    let Some(obj) = value.as_object() else {
        return map;
    };
    for (key, val) in obj {
        if let Some(b) = val.as_bool() {
            map.insert(key.clone(), b);
        }
    }
    map
}

fn validate_allowed_providers(providers: &[String]) -> Result<Vec<String>, (StatusCode, Json<ApiErrorBody>)> {
    if providers.is_empty() {
        return Err(bad_request("At least one AI provider must be allowed."));
    }
    let mut normalized = Vec::new();
    for p in providers {
        let trimmed = p.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.parse::<ProviderId>().is_err() {
            return Err(bad_request(format!("Unknown AI provider: {trimmed}")));
        }
        if normalized.iter().any(|existing: &String| existing == trimmed) {
            continue;
        }
        normalized.push(trimmed.to_string());
    }
    if normalized.is_empty() {
        return Err(bad_request("At least one AI provider must be allowed."));
    }
    Ok(normalized)
}

fn validate_default_limit(
    limit: Option<i64>,
) -> Result<Option<i64>, (StatusCode, Json<ApiErrorBody>)> {
    if let Some(value) = limit {
        if value <= 0 {
            return Err(bad_request(
                "Default monthly token limit must be greater than zero.",
            ));
        }
    }
    Ok(limit)
}

fn validate_announcement(
    announcement: Option<String>,
) -> Result<Option<String>, (StatusCode, Json<ApiErrorBody>)> {
    let Some(text) = announcement else {
        return Ok(None);
    };
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.len() > MAX_ANNOUNCEMENT_LEN {
        return Err(bad_request(format!(
            "Platform announcement must be at most {MAX_ANNOUNCEMENT_LEN} characters."
        )));
    }
    Ok(Some(trimmed.to_string()))
}

fn validate_feature_flags(
    flags: HashMap<String, bool>,
) -> Result<HashMap<String, bool>, (StatusCode, Json<ApiErrorBody>)> {
    for key in flags.keys() {
        if !is_valid_flag_key(key) {
            return Err(bad_request(
                "Feature flag keys may only contain letters, numbers, and underscores.",
            ));
        }
    }
    Ok(flags)
}

fn is_valid_flag_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn bad_request(message: impl Into<String>) -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::BAD_REQUEST,
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
