use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct Invitation {
    pub id: Uuid,
    #[allow(dead_code)]
    pub invitation_token_hash: String,
    pub company_id: Uuid,
    pub project_id: Option<Uuid>,
    pub invited_email: String,
    pub invited_role: String,
    pub invited_by_user_id: Uuid,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
