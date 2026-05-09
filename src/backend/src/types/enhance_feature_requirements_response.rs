use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnhanceFeatureRequirementsResponse {
    pub enhanced_requirements: String,
}
