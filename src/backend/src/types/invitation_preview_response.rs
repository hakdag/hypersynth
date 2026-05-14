use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::types::{CompanyRole, InvitationStatus};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationPreviewResponse {
    pub company_name: String,
    pub project_name: Option<String>,
    pub invited_role: CompanyRole,
    pub invited_email: String,
    pub status: InvitationStatus,
    pub expires_at: DateTime<Utc>,
    pub existing_user_present: bool,
}
