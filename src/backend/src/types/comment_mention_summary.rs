use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentMentionSummary {
    pub user_id: Uuid,
    pub username: String,
    pub fullname: String,
}
