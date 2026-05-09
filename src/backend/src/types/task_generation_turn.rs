use serde::{Deserialize, Serialize};

use crate::types::GeneratedTaskCandidate;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskGenerationTurn {
    pub proposed_tasks: Vec<GeneratedTaskCandidate>,
    pub feedback: String,
}
