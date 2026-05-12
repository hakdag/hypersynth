use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CompanyResponse {
    pub id: Uuid,
    pub name: String,
    pub company_email: String,
    pub country: String,
    pub timezone: String,
    pub legal_name: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub company_size: Option<String>,
    pub phone: Option<String>,
    pub billing_email: Option<String>,
    pub address: Option<String>,
    pub tax_vat_number: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
