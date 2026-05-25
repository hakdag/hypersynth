use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminAiUsageFailureQuery {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub company_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub provider: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
