use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub session_max_age_secs: i64,
    pub document_upload_dir: String,
}
