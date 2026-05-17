use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AdminCompanySummary {
    pub id: Uuid,
    pub name: String,
    pub company_email: String,
    pub status: String,
    pub user_count: i64,
    pub project_count: i64,
    pub document_count: i64,
    pub created_at: DateTime<Utc>,
}
