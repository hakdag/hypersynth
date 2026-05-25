use serde::Serialize;

use crate::types::HealthIndicatorStatus;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthIndicator {
    pub status: HealthIndicatorStatus,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}
