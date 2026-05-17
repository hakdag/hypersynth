use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserDetail {
    pub id: Uuid,
    pub full_name: String,
    pub display_name: Option<String>,
    pub email: String,
    pub username: Option<String>,
    pub account_type: String,
    pub role: Option<String>,
    pub status: String,
    pub timezone: Option<String>,
    pub company_id: Option<Uuid>,
    pub company_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sqlx(skip)]
    pub active_session_count: i64,
}
