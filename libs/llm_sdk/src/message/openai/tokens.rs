use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InputTokens {
    pub prompt_tokens: u32,
    pub completion_tokens: Option<u32>,
    pub total_tokens: u32,
}

/// Convert OpenAI token counting to Anthropic format
impl Into<crate::message::anthropic::tokens::InputTokens> for InputTokens {
    fn into(self) -> crate::message::anthropic::tokens::InputTokens {
        crate::message::anthropic::tokens::InputTokens {
            input_tokens: self.prompt_tokens,
        }
    }
}
