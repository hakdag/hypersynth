use std::env;

pub struct AnthropicConfig {
    pub base_url: String,
    pub model: String,
    pub max_tokens: u32,
    pub timeout_secs: u64,
}

impl AnthropicConfig {
    pub fn from_env() -> Result<Self, String> {
        let base_url =
            env::var("ANTHROPIC_BASE_URL").unwrap_or_else(|_| "https://api.anthropic.com".into());
        let model =
            env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-3-5-haiku-latest".into());
        let max_tokens: u32 = env::var("ANTHROPIC_MAX_TOKENS")
            .unwrap_or_else(|_| "2048".into())
            .parse()
            .map_err(|_| "ANTHROPIC_MAX_TOKENS must be a valid u32")?;
        let timeout_secs: u64 = env::var("ANTHROPIC_TIMEOUT_SECS")
            .unwrap_or_else(|_| "30".into())
            .parse()
            .map_err(|_| "ANTHROPIC_TIMEOUT_SECS must be a valid u64")?;

        if base_url.trim().is_empty() {
            return Err("ANTHROPIC_BASE_URL must not be empty".into());
        }
        if model.trim().is_empty() {
            return Err("ANTHROPIC_MODEL must not be empty".into());
        }
        if max_tokens == 0 {
            return Err("ANTHROPIC_MAX_TOKENS must be greater than 0".into());
        }
        if timeout_secs == 0 {
            return Err("ANTHROPIC_TIMEOUT_SECS must be greater than 0".into());
        }

        Ok(Self {
            base_url,
            model,
            max_tokens,
            timeout_secs,
        })
    }
}
