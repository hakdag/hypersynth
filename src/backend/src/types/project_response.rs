use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResponse {
    pub id: Uuid,
    pub owner_user_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
    pub name: String,
    pub requirements: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
