use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFeatureRequest {
    pub title: String,
    pub requirements: Option<String>,
}
