use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectRequest {
    pub name: String,
    pub requirements: String,
    pub status: String,
    #[serde(default)]
    pub clear_ai_api_key: bool,
    #[serde(default)]
    pub ai_api_key: Option<String>,
}
