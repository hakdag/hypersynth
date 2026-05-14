use serde::Serialize;

use super::invitation_status::InvitationStatus;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorBody {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invitation_status: Option<String>,
}

impl ApiErrorBody {
    pub fn msg(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            invitation_status: None,
        }
    }

    pub fn invitation_inactive(message: impl Into<String>, status: InvitationStatus) -> Self {
        Self {
            message: message.into(),
            invitation_status: Some(status.as_db_value().into()),
        }
    }
}

impl Default for ApiErrorBody {
    fn default() -> Self {
        Self {
            message: String::new(),
            invitation_status: None,
        }
    }
}
