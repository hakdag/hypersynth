use serde::Deserialize;

use crate::types::AiDocumentContextRequest;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnhanceProjectRequirementsRequest {
    #[serde(flatten, default)]
    pub document_context: AiDocumentContextRequest,
}

impl Default for EnhanceProjectRequirementsRequest {
    fn default() -> Self {
        Self {
            document_context: AiDocumentContextRequest::default(),
        }
    }
}
