use serde::Deserialize;

use crate::types::ProviderId;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAiModelsRequest {
    pub provider: ProviderId,
    pub api_key: String,
}
