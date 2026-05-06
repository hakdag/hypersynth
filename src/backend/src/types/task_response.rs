use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TaskResponse {
    pub id: Uuid,
    pub feature_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}
