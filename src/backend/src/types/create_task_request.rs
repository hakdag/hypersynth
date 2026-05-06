use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    /// When true, the task has no assignee (overrides assignee_user_id).
    #[serde(default)]
    pub unassigned: bool,
    #[serde(default)]
    pub assignee_user_id: Option<Uuid>,
}
