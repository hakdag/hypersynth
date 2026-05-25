use serde::Serialize;

use crate::types::ProviderId;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCatalogResponse {
    pub providers: Vec<ProviderId>,
}
