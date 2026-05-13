use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompanyRole {
    CompanyAdmin,
    ProjectManager,
    Contributor,
    Viewer,
}

impl CompanyRole {
    #[allow(dead_code)]
    pub fn as_db_value(self) -> &'static str {
        match self {
            CompanyRole::CompanyAdmin => "company_admin",
            CompanyRole::ProjectManager => "project_manager",
            CompanyRole::Contributor => "contributor",
            CompanyRole::Viewer => "viewer",
        }
    }

    pub fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "company_admin" => Some(CompanyRole::CompanyAdmin),
            "project_manager" => Some(CompanyRole::ProjectManager),
            "contributor" => Some(CompanyRole::Contributor),
            "viewer" => Some(CompanyRole::Viewer),
            _ => None,
        }
    }
}
