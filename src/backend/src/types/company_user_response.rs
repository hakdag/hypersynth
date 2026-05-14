use serde::Serialize;
use uuid::Uuid;

use crate::types::CompanyRole;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyUserResponse {
    pub id: Uuid,
    pub fullname: String,
    pub email: String,
    pub role: Option<CompanyRole>,
}
