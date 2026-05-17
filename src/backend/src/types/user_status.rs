use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Disabled,
    PendingInvitation,
}

impl UserStatus {
    pub fn as_db_value(self) -> &'static str {
        match self {
            UserStatus::Active => "active",
            UserStatus::Disabled => "disabled",
            UserStatus::PendingInvitation => "pending_invitation",
        }
    }

    pub fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "active" => Some(UserStatus::Active),
            "disabled" => Some(UserStatus::Disabled),
            "pending_invitation" => Some(UserStatus::PendingInvitation),
            _ => None,
        }
    }
}
