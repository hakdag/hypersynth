use sqlx::PgPool;

use crate::ai::AiProviderRegistry;
use crate::configs::{InvitationConfig, SystemAdminConfig};
use crate::email::EmailSender;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub session_max_age_secs: i64,
    pub document_upload_dir: String,
    pub api_key_encryption_key: [u8; 32],
    pub ai_providers: AiProviderRegistry,
    pub email_sender: std::sync::Arc<dyn EmailSender + Send + Sync>,
    pub invitation_config: InvitationConfig,
    pub system_admin: SystemAdminConfig,
}
