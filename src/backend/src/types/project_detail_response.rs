use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetailResponse {
    pub id: Uuid,
    pub owner_user_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
    pub name: String,
    pub requirements: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    #[sqlx(rename = "has_ai_api_key")]
    pub has_ai_api_key: bool,
    #[sqlx(rename = "can_manage_ai_settings")]
    pub can_manage_ai_settings: bool,
}
