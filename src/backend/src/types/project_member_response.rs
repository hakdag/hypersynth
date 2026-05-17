use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::types::{CompanyRole, ProjectMembershipRole};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMemberResponse {
    pub user_id: Uuid,
    pub fullname: String,
    pub email: String,
    pub company_role: Option<CompanyRole>,
    pub project_role: ProjectMembershipRole,
    pub created_at: DateTime<Utc>,
}
