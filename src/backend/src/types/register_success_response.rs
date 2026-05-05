use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterSuccessResponse {
    pub id: Uuid,
    pub message: String,
}
