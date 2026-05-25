use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminAuditLogEntry {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub company_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub system_admin_email: Option<String>,
    pub action_type: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub metadata: Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
