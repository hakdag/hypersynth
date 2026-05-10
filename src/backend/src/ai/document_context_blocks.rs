use serde_json::{json, Value};

use crate::types::{DocumentContentKind, DocumentContextItem};

pub fn build_document_context_blocks(items: &[DocumentContextItem]) -> Vec<Value> {
    let mut blocks = Vec::new();
    for item in items {
        match &item.kind {
            DocumentContentKind::Text(text) => {
                blocks.push(json!({
                    "type": "text",
                    "text": format!(
                        "Document: {}\n{}",
                        item.original_filename.as_str(),
                        text.as_str(),
                    ),
                }));
            }
            DocumentContentKind::Image {
                media_type,
                data_base64,
            } => {
                blocks.push(json!({
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": media_type,
                        "data": data_base64,
                    },
                }));
            }
        }
    }
    blocks
}
