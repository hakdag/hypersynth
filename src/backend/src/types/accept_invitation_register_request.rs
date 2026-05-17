use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptInvitationRegisterRequest {
    pub token: String,
    pub fullname: String,
    pub username: String,
    pub password: String,
    pub password_confirmation: String,
    pub timezone: Option<String>,
}
