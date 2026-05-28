use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CommentResponse {
    pub id: Uuid,
    pub task_id: Uuid,
    pub user_id: Uuid,
    pub author_fullname: String,
    pub author_avatar_url: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
