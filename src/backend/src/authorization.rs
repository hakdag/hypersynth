use axum::http::StatusCode;
use axum::Json;

use crate::types::{AccountType, ApiErrorBody, CompanyRole, SessionUser};

pub const MANAGE_COMPANY_PROFILE: &[CompanyRole] = &[CompanyRole::CompanyAdmin];

pub async fn require_company_role(
    user: &SessionUser,
    allowed: &[CompanyRole],
) -> Result<(), (StatusCode, Json<ApiErrorBody>)> {
    if user.account_type != AccountType::Company {
        return Err(forbidden(
            "You do not have permission to perform this action.",
        ));
    }

    let Some(role) = user.role else {
        return Err(forbidden(
            "You do not have permission to perform this action.",
        ));
    };

    if !allowed.contains(&role) {
        return Err(forbidden(
            "You do not have permission to perform this action.",
        ));
    }

    Ok(())
}

pub fn forbidden(message: impl Into<String>) -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::FORBIDDEN,
        Json(ApiErrorBody {
            message: message.into(),
        }),
    )
}
