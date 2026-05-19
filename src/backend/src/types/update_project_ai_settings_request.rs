use serde::Deserialize;

use crate::types::ProviderId;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectAiSettingsRequest {
    pub provider: ProviderId,
    pub allowed_models: Vec<String>,
    pub monthly_token_limit: Option<i64>,
    pub usage_tracking_enabled: bool,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub clear_api_key: bool,
}
