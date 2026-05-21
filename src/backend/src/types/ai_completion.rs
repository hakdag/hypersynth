use crate::types::AiTokenUsage;

#[derive(Debug)]
pub struct AiCompletion<T> {
    pub value: T,
    pub usage: AiTokenUsage,
}
