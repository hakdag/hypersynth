use sqlx::PgPool;

use crate::ai::AnthropicClient;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub session_max_age_secs: i64,
    pub document_upload_dir: String,
    pub api_key_encryption_key: [u8; 32],
    pub anthropic: AnthropicClient,
}
