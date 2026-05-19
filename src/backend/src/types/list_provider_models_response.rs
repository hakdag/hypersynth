use serde::Serialize;

use crate::types::ProviderId;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListProviderModelsResponse {
    pub provider: ProviderId,
    pub models: Vec<String>,
}
