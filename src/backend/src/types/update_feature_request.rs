use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFeatureRequest {
    pub title: String,
    pub requirements: Option<String>,
    pub status: String,
}
