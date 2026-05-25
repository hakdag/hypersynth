use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformConfig {
    pub allowed_ai_providers: Vec<String>,
    pub default_monthly_token_limit: Option<i64>,
    pub platform_announcement: Option<String>,
    pub feature_flags: HashMap<String, bool>,
    pub updated_at: DateTime<Utc>,
}
