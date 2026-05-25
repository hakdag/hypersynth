use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListProviderModelsRequest {
    pub api_key: String,
}
