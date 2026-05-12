use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCompanyRequest {
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
}
