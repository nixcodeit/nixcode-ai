use crate::models::llm_model::{
    DeepSeekR1, DeepSeekV3, Gemini25Pro, Gpt4o, Haiku35, LLMModel, Llama4, Llama4OpenRouter,
    QuasarAlpha, QwenQwq32b, Sonnet37,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub enum LLMProvider {
    #[default]
    Anthropic,
    OpenAI,
    Gemini,
    Groq,
    OpenRouter,
}

impl LLMProvider {
    pub fn name(&self) -> &'static str {
        match self {
            LLMProvider::Anthropic => "Anthropic",
            LLMProvider::OpenAI => "OpenAI",
            LLMProvider::Gemini => "Gemini",
            LLMProvider::Groq => "Groq",
            LLMProvider::OpenRouter => "OpenRouter",
        }
    }

    pub fn default_model(&self) -> &'static LLMModel {
        match self {
            LLMProvider::Anthropic => &Sonnet37,
            LLMProvider::OpenAI => &Gpt4o,
            LLMProvider::Groq => &DeepSeekR1,
            LLMProvider::OpenRouter => &DeepSeekV3,
            _ => panic!("No default model for provider: {}", self.name()),
        }
    }
}
