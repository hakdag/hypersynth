use serde::Deserialize;

use crate::types::TaskGenerationTurn;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateTasksRequest {
    #[serde(default)]
    pub feedback_history: Vec<TaskGenerationTurn>,
}
