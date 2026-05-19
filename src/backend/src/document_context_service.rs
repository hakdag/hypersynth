use std::collections::{HashMap, HashSet};

use base64::engine::general_purpose::STANDARD as B64_STD;
use base64::Engine;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::types::{DocumentContentKind, DocumentContextError, DocumentContextItem, TenantScope};

pub struct DocumentContextService;

impl DocumentContextService {
    pub async fn load_for_project(
        pool: &PgPool,
        project_id: Uuid,
        scope: TenantScope,
        document_ids: &[Uuid],
    ) -> Result<Vec<DocumentContextItem>, DocumentContextError> {
        let ordered = Self::dedupe_preserve_order(document_ids);
        if ordered.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query_as::<_, (Uuid, String, Value)>(
            r#"
            SELECT d.id, d.file_path, d.metadata
            FROM project_documents d
            INNER JOIN projects p ON p.id = d.project_id
            WHERE d.project_id = $1
              AND (
                ($3::uuid IS NOT NULL AND p.owner_user_id = $3 AND p.company_id IS NULL)
                OR
                ($2::uuid IS NOT NULL AND p.company_id = $2 AND (
                    $4::boolean
                    OR EXISTS (
                        SELECT 1 FROM project_memberships pm
                        WHERE pm.project_id = p.id AND pm.user_id = $5
                    )
                ))
              )
              AND d.id = ANY($6)
            "#,
        )
        .bind(project_id)
        .bind(scope.company_id_or_null())
        .bind(scope.owner_user_id_or_null())
        .bind(scope.is_company_admin())
        .bind(scope.session_user_id())
        .bind(&ordered)
        .fetch_all(pool)
        .await
        .map_err(|_| DocumentContextError::ContentUnavailable)?;

        if rows.len() != ordered.len() {
            return Err(DocumentContextError::NotFoundOrForbidden);
        }

        let mut by_id: HashMap<Uuid, (Uuid, String, Value)> = HashMap::with_capacity(rows.len());
        for (id, path, meta) in rows {
            by_id.insert(id, (id, path, meta));
        }

        let mut out = Vec::with_capacity(ordered.len());
        for id in ordered {
            let row = by_id
                .remove(&id)
                .ok_or(DocumentContextError::NotFoundOrForbidden)?;
            let item = Self::load_one(row).await?;
            out.push(item);
        }
        Ok(out)
    }

    fn dedupe_preserve_order(ids: &[Uuid]) -> Vec<Uuid> {
        let mut seen = HashSet::new();
        ids.iter().copied().filter(|u| seen.insert(*u)).collect()
    }

    async fn load_one(
        row: (Uuid, String, Value),
    ) -> Result<DocumentContextItem, DocumentContextError> {
        let (doc_id, file_path, metadata) = row;
        let original_filename = metadata_as_str(&metadata, "originalFilename")
            .unwrap_or_else(|| "document".to_string());
        let mime_raw = metadata_as_str(&metadata, "contentType").unwrap_or_default();
        let mime = mime_raw
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();

        let bytes = tokio::fs::read(&file_path)
            .await
            .map_err(|_| DocumentContextError::ContentUnavailable)?;

        let kind = classify_document(&mime, &bytes)?;
        Ok(DocumentContextItem {
            id: doc_id,
            original_filename,
            mime,
            kind,
        })
    }
}

fn metadata_as_str(metadata: &Value, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
}

fn sniff_image_media_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() >= 3 && bytes[0..3] == [0xFF, 0xD8, 0xFF] {
        return Some("image/jpeg");
    }
    if bytes.len() >= 8 && bytes[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return Some("image/png");
    }
    if bytes.len() >= 6 && (&bytes[0..6] == b"GIF87a" || &bytes[0..6] == b"GIF89a") {
        return Some("image/gif");
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    None
}

fn classify_document(
    mime: &str,
    bytes: &[u8],
) -> Result<DocumentContentKind, DocumentContextError> {
    if mime.starts_with("text/")
        || mime == "application/json"
        || mime == "text/csv"
        || mime == "application/csv"
    {
        let text = std::str::from_utf8(bytes)
            .map_err(|_| DocumentContextError::ContentUnavailable)?
            .to_string();
        return Ok(DocumentContentKind::Text(text));
    }

    let image_media = match mime {
        "image/png" => Some("image/png"),
        "image/jpeg" | "image/jpg" => Some("image/jpeg"),
        "image/gif" => Some("image/gif"),
        "image/webp" => Some("image/webp"),
        "" | "application/octet-stream" => sniff_image_media_type(bytes),
        m if m.starts_with("image/") => return Err(DocumentContextError::UnsupportedDocumentType),
        _ => None,
    };

    if let Some(media_type) = image_media {
        let data_base64 = B64_STD.encode(bytes);
        return Ok(DocumentContentKind::Image {
            media_type: media_type.to_string(),
            data_base64,
        });
    }

    if mime.is_empty() || mime == "application/octet-stream" {
        if let Ok(text) = std::str::from_utf8(bytes) {
            return Ok(DocumentContentKind::Text(text.to_string()));
        }
    }

    Err(DocumentContextError::UnsupportedDocumentType)
}
