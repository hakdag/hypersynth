use serde::Deserialize;

use crate::types::GeneratedTaskCandidate;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptGeneratedTasksRequest {
    pub tasks: Vec<GeneratedTaskCandidate>,
}
