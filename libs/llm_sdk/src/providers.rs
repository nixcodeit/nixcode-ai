use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum LLMProvider {
    Anthropic,
    OpenAI,
    Gemini,
    Groq,
}

impl LLMProvider {
    pub fn default_model(&self) -> &str {
        match self {
            LLMProvider::Anthropic => "claude-3-7-sonnet-20250219",
            LLMProvider::OpenAI => "gpt-4o",
            LLMProvider::Gemini => "gemini-pro",
            LLMProvider::Groq => "qwen-qwq-32b",
        }
    }

    pub fn available_models(&self) -> Vec<&str> {
        match self {
            LLMProvider::Anthropic => vec![
                "claude-3-opus-20240229",
                "claude-3-sonnet-20240229",
                "claude-3-haiku-20240307",
                "claude-3-7-sonnet-20250219",
            ],
            LLMProvider::OpenAI => vec!["gpt-4o", "gpt-4-turbo", "gpt-4", "gpt-3.5-turbo"],
            LLMProvider::Gemini => vec!["gemini-pro", "gemini-ultra"],
            LLMProvider::Groq => vec!["qwen-qwq-32b", "qwen-2.5-coder-32b", "llama-3.3-70b-specdec"],
        }
    }
}
