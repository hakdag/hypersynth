use serde::Deserialize;
use uuid::Uuid;

use crate::types::CompanyRole;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInvitationRequest {
    pub invited_email: String,
    pub invited_role: CompanyRole,
    pub project_id: Option<Uuid>,
    pub message: Option<String>,
}
