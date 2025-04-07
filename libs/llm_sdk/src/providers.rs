use crate::models::llm_model::{Gpt4o, Haiku35, LLMModel, Llama4};
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
            LLMProvider::Anthropic => &Haiku35,
            LLMProvider::OpenAI => &Gpt4o,
            LLMProvider::Groq => &Llama4,
            _ => panic!("No default model for provider: {}", self.name()),
        }
    }
}
