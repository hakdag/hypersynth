use std::collections::HashMap;

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapResponse {
    pub app_name: &'static str,
    pub status_labels: [&'static str; 3],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_announcement: Option<String>,
    pub feature_flags: HashMap<String, bool>,
}
