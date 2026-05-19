use serde::Serialize;
use uuid::Uuid;

use crate::types::ProviderId;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAiSettingsResponse {
    pub project_id: Uuid,
    pub provider: Option<ProviderId>,
    pub allowed_models: Vec<String>,
    pub monthly_token_limit: Option<i64>,
    pub usage_tracking_enabled: bool,
    pub has_api_key: bool,
    pub api_key_hint: Option<String>,
}
