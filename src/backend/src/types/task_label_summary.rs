use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskLabelSummary {
    pub id: Uuid,
    pub name: String,
    pub color: String,
}
