use serde::Serialize;

use super::admin_invitation_summary::AdminInvitationSummary;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminInvitationsListResponse {
    pub items: Vec<AdminInvitationSummary>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
