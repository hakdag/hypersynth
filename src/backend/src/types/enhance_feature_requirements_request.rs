use serde::Deserialize;

use crate::types::AiDocumentContextRequest;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnhanceFeatureRequirementsRequest {
    #[serde(flatten, default)]
    pub document_context: AiDocumentContextRequest,
}

impl Default for EnhanceFeatureRequirementsRequest {
    fn default() -> Self {
        Self {
            document_context: AiDocumentContextRequest::default(),
        }
    }
}
