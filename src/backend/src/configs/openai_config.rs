use std::env;

pub struct OpenAiConfig {
    pub base_url: String,
    pub default_model: String,
    pub max_tokens: u32,
    pub timeout_secs: u64,
}

impl OpenAiConfig {
    pub fn from_env() -> Result<Self, String> {
        let base_url =
            env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com".into());
        let default_model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".into());
        let max_tokens: u32 = env::var("OPENAI_MAX_TOKENS")
            .unwrap_or_else(|_| "2048".into())
            .parse()
            .map_err(|_| "OPENAI_MAX_TOKENS must be a valid u32")?;
        let timeout_secs: u64 = env::var("OPENAI_TIMEOUT_SECS")
            .unwrap_or_else(|_| "30".into())
            .parse()
            .map_err(|_| "OPENAI_TIMEOUT_SECS must be a valid u64")?;

        if base_url.trim().is_empty() {
            return Err("OPENAI_BASE_URL must not be empty".into());
        }
        if default_model.trim().is_empty() {
            return Err("OPENAI_MODEL must not be empty".into());
        }
        if max_tokens == 0 {
            return Err("OPENAI_MAX_TOKENS must be greater than 0".into());
        }
        if timeout_secs == 0 {
            return Err("OPENAI_TIMEOUT_SECS must be greater than 0".into());
        }

        Ok(Self {
            base_url,
            default_model,
            max_tokens,
            timeout_secs,
        })
    }
}
