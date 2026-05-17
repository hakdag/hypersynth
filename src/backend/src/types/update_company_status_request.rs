use serde::Deserialize;

use crate::types::CompanyStatus;

#[derive(Debug, Deserialize)]
pub struct UpdateCompanyStatusRequest {
    pub status: CompanyStatus,
}
