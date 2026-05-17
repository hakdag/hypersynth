use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminAiUsageSummary {
    pub total_requests: i64,
    pub total_tokens: i64,
    pub estimated_cost: f64,
}
