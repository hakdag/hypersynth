use uuid::Uuid;

use crate::types::DocumentContentKind;

#[derive(Debug)]
pub struct DocumentContextItem {
    /// Stored for correctness checks and future auditing; omitted from prompts.
    #[allow(dead_code)]
    pub id: Uuid,
    pub original_filename: String,
    #[allow(dead_code)]
    pub mime: String,
    pub kind: DocumentContentKind,
}
