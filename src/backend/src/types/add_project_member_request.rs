use serde::Deserialize;
use uuid::Uuid;

use crate::types::ProjectMembershipRole;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddProjectMemberRequest {
    pub user_id: Uuid,
    pub project_role: ProjectMembershipRole,
}
