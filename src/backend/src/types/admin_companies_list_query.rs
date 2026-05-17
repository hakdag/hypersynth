use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AdminCompaniesListQuery {
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
