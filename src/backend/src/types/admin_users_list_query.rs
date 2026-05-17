use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct AdminUsersListQuery {
    pub search: Option<String>,
    pub account_type: Option<String>,
    pub status: Option<String>,
    pub company_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
