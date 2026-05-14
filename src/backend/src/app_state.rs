use sqlx::PgPool;

use crate::ai::AnthropicClient;
use crate::configs::InvitationConfig;
use crate::email::EmailSender;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub session_max_age_secs: i64,
    pub document_upload_dir: String,
    pub api_key_encryption_key: [u8; 32],
    pub anthropic: AnthropicClient,
    pub email_sender: std::sync::Arc<dyn EmailSender + Send + Sync>,
    pub invitation_config: InvitationConfig,
}
