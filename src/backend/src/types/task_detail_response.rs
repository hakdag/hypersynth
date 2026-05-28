use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TaskDetailResponse {
    pub id: Uuid,
    pub feature_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub priority: String,
    pub due_date: Option<NaiveDate>,
    pub due_time: Option<NaiveTime>,
    #[sqlx(default)]
    pub is_overdue: bool,
    pub assignee_user_id: Option<Uuid>,
    pub assignee_fullname: Option<String>,
    pub assignee_avatar_url: Option<String>,
    pub creator_fullname: Option<String>,
    pub creator_avatar_url: Option<String>,
    pub feature_title: String,
    pub project_id: Uuid,
    pub project_name: String,
}
