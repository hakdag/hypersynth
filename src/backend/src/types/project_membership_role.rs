use serde::{Deserialize, Serialize};

use crate::types::CompanyRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectMembershipRole {
    ProjectManager,
    Contributor,
    Viewer,
}

impl ProjectMembershipRole {
    pub fn as_db_value(self) -> &'static str {
        match self {
            ProjectMembershipRole::ProjectManager => "project_manager",
            ProjectMembershipRole::Contributor => "contributor",
            ProjectMembershipRole::Viewer => "viewer",
        }
    }

    pub fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "project_manager" => Some(ProjectMembershipRole::ProjectManager),
            "contributor" => Some(ProjectMembershipRole::Contributor),
            "viewer" => Some(ProjectMembershipRole::Viewer),
            _ => None,
        }
    }

    /// Maps invited company role to project membership role. Company admins are not stored as members.
    pub fn from_company_role(role: CompanyRole) -> Option<Self> {
        match role {
            CompanyRole::CompanyAdmin => None,
            CompanyRole::ProjectManager => Some(ProjectMembershipRole::ProjectManager),
            CompanyRole::Contributor => Some(ProjectMembershipRole::Contributor),
            CompanyRole::Viewer => Some(ProjectMembershipRole::Viewer),
        }
    }
}
