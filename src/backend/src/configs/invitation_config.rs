use std::env;

#[derive(Clone)]
pub struct InvitationConfig {
    pub expires_in_hours: i64,
    pub app_base_url: String,
}

impl InvitationConfig {
    pub fn from_env() -> Result<Self, String> {
        let expires_in_hours: i64 = env::var("INVITATION_EXPIRES_IN_HOURS")
            .unwrap_or_else(|_| "168".into())
            .parse()
            .map_err(|_| "INVITATION_EXPIRES_IN_HOURS must be a valid i64 (hours)")?;

        if expires_in_hours < 1 {
            return Err("INVITATION_EXPIRES_IN_HOURS must be at least 1".into());
        }

        let app_base_url = env::var("APP_BASE_URL").map_err(|_| {
            "APP_BASE_URL is required (e.g. http://localhost:4200 — no trailing slash)".to_string()
        })?;

        let trimmed = app_base_url.trim();
        if trimmed.is_empty() {
            return Err("APP_BASE_URL must not be empty".into());
        }

        let normalized = trimmed.trim_end_matches('/').to_string();

        Ok(Self {
            expires_in_hours,
            app_base_url: normalized,
        })
    }
}
