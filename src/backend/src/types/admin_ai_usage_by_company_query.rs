use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::admin_ai_usage_high_usage_sort::AdminAiUsageHighUsageSort;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminAiUsageByCompanyQuery {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    #[serde(default)]
    pub sort: AdminAiUsageHighUsageSort,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
