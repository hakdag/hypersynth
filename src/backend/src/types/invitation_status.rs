use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Expired,
    Cancelled,
}

impl InvitationStatus {
    pub fn as_db_value(self) -> &'static str {
        match self {
            InvitationStatus::Pending => "pending",
            InvitationStatus::Accepted => "accepted",
            InvitationStatus::Expired => "expired",
            InvitationStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(InvitationStatus::Pending),
            "accepted" => Some(InvitationStatus::Accepted),
            "expired" => Some(InvitationStatus::Expired),
            "cancelled" => Some(InvitationStatus::Cancelled),
            _ => None,
        }
    }
}
