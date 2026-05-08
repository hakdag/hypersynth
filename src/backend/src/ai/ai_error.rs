use axum::http::StatusCode;
use std::fmt;

#[derive(Debug)]
pub enum AiError {
    Network,
    Provider(StatusCode),
    Decode,
    Empty,
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiError::Network => f.write_str("failed to call AI provider"),
            AiError::Provider(status) => write!(f, "ai provider returned status {}", status),
            AiError::Decode => f.write_str("failed to decode AI provider response"),
            AiError::Empty => f.write_str("ai provider returned an empty response"),
        }
    }
}

impl std::error::Error for AiError {}
