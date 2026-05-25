use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthIndicatorStatus {
    Healthy,
    Degraded,
    Unavailable,
    NotConfigured,
}
