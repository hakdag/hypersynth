use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlatformConfigRequest {
    pub allowed_ai_providers: Option<Vec<String>>,
    pub default_monthly_token_limit: Option<Option<i64>>,
    pub platform_announcement: Option<Option<String>>,
    pub feature_flags: Option<HashMap<String, bool>>,
}
