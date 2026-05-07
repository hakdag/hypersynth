use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    /// When true, the task has no assignee (overrides assignee_user_id).
    #[serde(default)]
    pub unassigned: bool,
    #[serde(default)]
    pub assignee_user_id: Option<Uuid>,
}
