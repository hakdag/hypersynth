use serde::Serialize;

use crate::types::GeneratedTaskCandidate;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateTasksResponse {
    pub tasks: Vec<GeneratedTaskCandidate>,
}
