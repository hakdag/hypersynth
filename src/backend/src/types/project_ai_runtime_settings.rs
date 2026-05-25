use crate::types::ProviderId;

#[derive(Debug)]
pub struct ProjectAiRuntimeSettings {
    pub provider: ProviderId,
    pub api_key: String,
    pub selected_model: String,
    #[allow(dead_code)]
    pub usage_tracking_enabled: bool,
}
