use crate::models::llm_model::LLMModel;
use crate::providers::LLMProvider;
use secrecy::SecretString;

#[derive(Clone)]
pub struct LLMConfig {
    pub provider: LLMProvider,
    pub api_key: SecretString,
    pub default_model: &'static LLMModel,
    pub api_base: Option<String>,
}

impl LLMConfig {
    pub fn new_anthropic(api_key: SecretString) -> Self {
        Self {
            provider: LLMProvider::Anthropic,
            api_key,
            default_model: LLMProvider::Anthropic.default_model(),
            api_base: None,
        }
    }

    pub fn new_openai(api_key: SecretString) -> Self {
        Self {
            provider: LLMProvider::OpenAI,
            api_key,
            default_model: LLMProvider::OpenAI.default_model(),
            api_base: Some("https://api.openai.com".to_string()),
        }
    }

    pub fn new_groq(api_key: SecretString) -> Self {
        Self {
            provider: LLMProvider::Groq,
            api_key,
            default_model: LLMProvider::Groq.default_model(),
            api_base: Some("https://api.groq.com/openai".to_string()),
        }
    }

    pub fn new_openrouter(api_key: SecretString) -> Self {
        Self {
            provider: LLMProvider::OpenRouter,
            api_key,
            default_model: LLMProvider::OpenRouter.default_model(),
            api_base: Some("https://openrouter.ai/api".to_string()),
        }
    }
}
