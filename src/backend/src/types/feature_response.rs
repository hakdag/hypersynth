use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct FeatureResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub requirements: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
