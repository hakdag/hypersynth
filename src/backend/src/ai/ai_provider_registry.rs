use std::sync::Arc;

use crate::ai::{AiProvider, AnthropicProvider, OpenAiProvider};
use crate::types::ProviderId;

#[derive(Clone)]
pub struct AiProviderRegistry {
    anthropic: Arc<AnthropicProvider>,
    openai: Arc<OpenAiProvider>,
}

impl AiProviderRegistry {
    pub fn new(anthropic: AnthropicProvider, openai: OpenAiProvider) -> Self {
        Self {
            anthropic: Arc::new(anthropic),
            openai: Arc::new(openai),
        }
    }

    pub fn supported(&self) -> Vec<ProviderId> {
        vec![self.anthropic.id(), self.openai.id()]
    }

    pub fn get(&self, id: ProviderId) -> &dyn AiProvider {
        match id {
            ProviderId::Anthropic => self.anthropic.as_ref(),
            ProviderId::OpenAi => self.openai.as_ref(),
        }
    }
}
