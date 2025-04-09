use crate::models::llm_model::{DeepSeekV3, Gpt3oMini, LLMModel, Llama4, Sonnet37};
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
    pub fn config_key(&self) -> &'static str {
        match self {
            LLMProvider::Anthropic => "anthropic",
            LLMProvider::OpenAI => "openai",
            LLMProvider::Gemini => "gemini",
            LLMProvider::Groq => "groq",
            LLMProvider::OpenRouter => "open_router",
        }
    }

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
            LLMProvider::OpenAI => &Gpt3oMini,
            LLMProvider::Groq => &Llama4,
            LLMProvider::OpenRouter => &DeepSeekV3,
            _ => panic!("No default model for provider: {}", self.name()),
        }
    }
}
