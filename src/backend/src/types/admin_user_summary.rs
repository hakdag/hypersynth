use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserSummary {
    pub id: Uuid,
    pub full_name: String,
    pub email: String,
    pub username: Option<String>,
    pub account_type: String,
    pub role: Option<String>,
    pub status: String,
    pub company_id: Option<Uuid>,
    pub company_name: Option<String>,
    pub created_at: DateTime<Utc>,
}
