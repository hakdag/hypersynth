use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    Anthropic,
    #[serde(rename = "openai")]
    OpenAi,
}

impl ProviderId {
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderId::Anthropic => "anthropic",
            ProviderId::OpenAi => "openai",
        }
    }
}

impl FromStr for ProviderId {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "anthropic" => Ok(ProviderId::Anthropic),
            "openai" => Ok(ProviderId::OpenAi),
            _ => Err(()),
        }
    }
}
