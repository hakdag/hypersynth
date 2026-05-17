use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompanyStatus {
    Active,
    Disabled,
    PendingVerification,
}

impl CompanyStatus {
    pub fn as_db_value(self) -> &'static str {
        match self {
            CompanyStatus::Active => "active",
            CompanyStatus::Disabled => "disabled",
            CompanyStatus::PendingVerification => "pending_verification",
        }
    }

    #[allow(dead_code)]
    pub fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "active" => Some(CompanyStatus::Active),
            "disabled" => Some(CompanyStatus::Disabled),
            "pending_verification" => Some(CompanyStatus::PendingVerification),
            _ => None,
        }
    }
}
