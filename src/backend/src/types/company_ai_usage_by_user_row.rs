use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CompanyAiUsageByUserRow {
    pub user_id: Uuid,
    pub user_email: String,
    pub user_full_name: String,
    pub request_count: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub estimated_cost: f64,
    pub success_count: i64,
    pub failure_count: i64,
}
