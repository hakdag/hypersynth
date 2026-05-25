use serde::Serialize;

use crate::types::HealthIndicator;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminSystemHealthResponse {
    pub application: HealthIndicator,
    pub database: HealthIndicator,
    pub background_jobs: HealthIndicator,
    pub ai_provider_error_rate: HealthIndicator,
    pub email_delivery: HealthIndicator,
    pub storage: HealthIndicator,
}
