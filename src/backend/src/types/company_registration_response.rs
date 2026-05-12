use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyRegistrationResponse {
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub message: String,
}
