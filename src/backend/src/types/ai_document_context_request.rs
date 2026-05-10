use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiDocumentContextRequest {
    #[serde(default)]
    pub document_ids: Vec<Uuid>,
}

impl Default for AiDocumentContextRequest {
    fn default() -> Self {
        Self {
            document_ids: Vec::new(),
        }
    }
}
