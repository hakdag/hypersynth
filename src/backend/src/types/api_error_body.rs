use serde::Serialize;

use super::invitation_status::InvitationStatus;

pub const ERROR_CODE_COMPANY_DISABLED: &str = "company_disabled";
pub const ERROR_CODE_USER_DISABLED: &str = "user_disabled";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorBody {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invitation_status: Option<String>,
}

impl ApiErrorBody {
    pub fn msg(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: None,
            invitation_status: None,
        }
    }

    pub fn company_disabled() -> Self {
        Self {
            message: "Your company account has been disabled. Please contact your administrator."
                .into(),
            code: Some(ERROR_CODE_COMPANY_DISABLED.into()),
            invitation_status: None,
        }
    }

    pub fn user_disabled() -> Self {
        Self {
            message: "Your account has been disabled. Please contact support.".into(),
            code: Some(ERROR_CODE_USER_DISABLED.into()),
            invitation_status: None,
        }
    }

    pub fn invitation_inactive(message: impl Into<String>, status: InvitationStatus) -> Self {
        Self {
            message: message.into(),
            code: None,
            invitation_status: Some(status.as_db_value().into()),
        }
    }
}

impl Default for ApiErrorBody {
    fn default() -> Self {
        Self {
            message: String::new(),
            code: None,
            invitation_status: None,
        }
    }
}
