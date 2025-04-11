use crate::models::llm_model::{DeepSeekV3, Gemini20Flash, Gpt3oMini, LLMModel, Llama4, Sonnet37};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub enum LLMProvider {
    #[default]
    Anthropic,
    OpenAI,
    Gemini,
    Groq,
    OpenRouter,
    GenAI,
    Llama,
}

impl LLMProvider {
    pub fn config_key(&self) -> &'static str {
        match self {
            LLMProvider::Anthropic => "anthropic",
            LLMProvider::OpenAI => "openai",
            LLMProvider::Gemini => "gemini",
            LLMProvider::Groq => "groq",
            LLMProvider::OpenRouter => "open_router",
            LLMProvider::GenAI => "genai",
            LLMProvider::Llama => "llama",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LLMProvider::Anthropic => "Anthropic",
            LLMProvider::OpenAI => "OpenAI",
            LLMProvider::Gemini => "Gemini",
            LLMProvider::Groq => "Groq",
            LLMProvider::OpenRouter => "OpenRouter",
            LLMProvider::GenAI => "GenAI",
            LLMProvider::Llama => "Llama",
        }
    }

    pub fn default_model(&self) -> Arc<LLMModel> {
        match self {
            LLMProvider::Anthropic => Sonnet37.clone(),
            LLMProvider::OpenAI => Gpt3oMini.clone(),
            LLMProvider::Groq => Llama4.clone(),
            LLMProvider::OpenRouter => DeepSeekV3.clone(),
            LLMProvider::GenAI => Gemini20Flash.clone(),
            _ => panic!("No default model for provider: {}", self.name()),
        }
    }
}
