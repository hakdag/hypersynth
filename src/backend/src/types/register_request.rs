use serde::Deserialize;

use crate::types::AccountType;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub account_type: AccountType,
    pub fullname: String,
    pub email: String,
    pub password: String,
}
