use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AdminInvitationSummary {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub invited_by_user_id: Uuid,
    pub inviter_name: String,
    pub inviter_email: String,
    pub invited_email: String,
    pub invited_role: String,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
