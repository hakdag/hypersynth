use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyRegistrationRequest {
    pub name: String,
    pub company_email: String,
    pub country: String,
    pub timezone: String,
    pub full_name: String,
    pub email: String,
    pub username: String,
    pub password: String,
    pub password_confirmation: String,
}
