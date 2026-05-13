use uuid::Uuid;

use crate::types::{AccountType, CompanyRole};

#[derive(Debug)]
pub struct SessionUser {
    pub id: Uuid,
    pub fullname: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub account_type: AccountType,
    pub role: Option<CompanyRole>,
    pub company_id: Option<Uuid>,
}
