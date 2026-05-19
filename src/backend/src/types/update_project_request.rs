use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectRequest {
    pub name: String,
    pub requirements: String,
    pub status: String,
}
