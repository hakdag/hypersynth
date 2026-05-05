use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapResponse {
    pub app_name: &'static str,
    pub status_labels: [&'static str; 3],
}
