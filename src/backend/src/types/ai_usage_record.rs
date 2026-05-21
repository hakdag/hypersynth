use uuid::Uuid;

use crate::types::{AiOperationType, AiUsageStatus, ProviderId};

#[derive(Debug)]
pub struct AiUsageRecord {
    pub company_id: Option<Uuid>,
    pub user_id: Uuid,
    pub project_id: Option<Uuid>,
    pub feature_id: Option<Uuid>,
    pub operation_type: AiOperationType,
    pub provider: ProviderId,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub estimated_cost_micros: i64,
    pub status: AiUsageStatus,
    pub error_code: Option<String>,
}
