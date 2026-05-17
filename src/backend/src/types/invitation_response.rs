use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::types::{CompanyRole, InvitationStatus};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationResponse {
    pub id: Uuid,
    pub company_id: Uuid,
    pub project_id: Option<Uuid>,
    pub invited_email: String,
    pub invited_role: CompanyRole,
    pub invited_by_user_id: Uuid,
    pub status: InvitationStatus,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
