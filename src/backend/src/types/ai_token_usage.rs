#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AiTokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}
