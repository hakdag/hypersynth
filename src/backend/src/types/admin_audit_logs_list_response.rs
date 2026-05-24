use serde::Serialize;

use super::admin_audit_log_entry::AdminAuditLogEntry;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminAuditLogsListResponse {
    pub items: Vec<AdminAuditLogEntry>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
