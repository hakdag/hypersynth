use uuid::Uuid;

use crate::types::{AiOperationType, ProviderId};

#[derive(Debug, Clone, Copy)]
pub struct AiUsageScope<'a> {
    pub company_id: Option<Uuid>,
    pub user_id: Uuid,
    pub project_id: Option<Uuid>,
    pub feature_id: Option<Uuid>,
    pub operation_type: AiOperationType,
    pub provider: ProviderId,
    pub model: &'a str,
}
