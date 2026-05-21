#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiUsageStatus {
    Success,
    Failed,
}

impl AiUsageStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            AiUsageStatus::Success => "success",
            AiUsageStatus::Failed => "failed",
        }
    }
}
