use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnhanceProjectRequirementsResponse {
    pub enhanced_requirements: String,
}
