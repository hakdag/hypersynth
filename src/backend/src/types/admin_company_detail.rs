use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::types::AdminAiUsageSummary;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AdminCompanyDetail {
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
    pub user_count: i64,
    pub project_count: i64,
    pub document_count: i64,
    #[sqlx(skip)]
    pub ai_usage: Option<AdminAiUsageSummary>,
}
