use std::env;

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;

use crate::configs::{AnthropicConfig, InvitationConfig, SmtpConfig, SystemAdminConfig};

const API_KEY_ENCRYPTION_KEY_LEN: usize = 32;

/// Application configuration loaded from the environment.
pub struct AppConfig {
    pub port: u16,
    pub database_url: String,
    pub cors_origin: String,
    pub session_max_age_secs: i64,
    pub document_upload_dir: String,
    pub api_key_encryption_key: [u8; API_KEY_ENCRYPTION_KEY_LEN],
    pub anthropic_config: AnthropicConfig,
    pub invitation_config: InvitationConfig,
    pub smtp_config: SmtpConfig,
    pub system_admin_config: SystemAdminConfig,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        let port: u16 = env::var("PORT")
            .unwrap_or_else(|_| "3000".into())
            .parse()
            .map_err(|_| "PORT must be a valid u16")?;

        let database_url = env::var("DATABASE_URL").map_err(|_| {
            "DATABASE_URL is required (e.g. postgres://user:pass@localhost:5432/hypersynth)"
        })?;

        let cors_origin =
            env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:4200".into());

        let session_max_age_secs: i64 = env::var("SESSION_MAX_AGE_SECS")
            .unwrap_or_else(|_| "604800".into())
            .parse()
            .map_err(|_| "SESSION_MAX_AGE_SECS must be a valid i64 (seconds)")?;

        if session_max_age_secs < 60 {
            return Err("SESSION_MAX_AGE_SECS must be at least 60".into());
        }

        let document_upload_dir =
            env::var("DOCUMENT_UPLOAD_DIR").unwrap_or_else(|_| "./uploaded-documents".into());

        let api_key_encryption_key = parse_api_key_encryption_key()?;
        let anthropic_config = AnthropicConfig::from_env()?;
        let invitation_config = InvitationConfig::from_env()?;
        let smtp_config = SmtpConfig::from_env()?;
        let system_admin_config = SystemAdminConfig::from_env()?;

        Ok(Self {
            port,
            database_url,
            cors_origin,
            session_max_age_secs,
            document_upload_dir,
            api_key_encryption_key,
            anthropic_config,
            invitation_config,
            smtp_config,
            system_admin_config,
        })
    }
}

fn parse_api_key_encryption_key() -> Result<[u8; API_KEY_ENCRYPTION_KEY_LEN], String> {
    let raw = env::var("API_KEY_ENCRYPTION_KEY").map_err(|_| {
        "API_KEY_ENCRYPTION_KEY is required (base64-encoded 32 random bytes; \
         generate with `openssl rand -base64 32`)"
            .to_string()
    })?;

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("API_KEY_ENCRYPTION_KEY must not be empty".into());
    }

    let decoded = BASE64_STANDARD
        .decode(trimmed)
        .map_err(|_| "API_KEY_ENCRYPTION_KEY must be valid base64".to_string())?;

    if decoded.len() != API_KEY_ENCRYPTION_KEY_LEN {
        return Err(format!(
            "API_KEY_ENCRYPTION_KEY must decode to exactly {} bytes (got {})",
            API_KEY_ENCRYPTION_KEY_LEN,
            decoded.len()
        ));
    }

    let mut bytes = [0u8; API_KEY_ENCRYPTION_KEY_LEN];
    bytes.copy_from_slice(&decoded);
    Ok(bytes)
}
