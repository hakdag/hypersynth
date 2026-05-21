use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CompanyAiUsageFailureRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub provider: String,
    pub model: String,
    pub operation_type: String,
    pub error_code: Option<String>,
    pub created_at: DateTime<Utc>,
}
