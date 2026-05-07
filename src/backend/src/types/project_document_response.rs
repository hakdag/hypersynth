use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDocumentResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub file_path: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}
