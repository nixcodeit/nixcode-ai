use crate::providers::LLMProvider;
use secrecy::SecretString;

#[derive(Debug, Clone)]
pub struct LLMConfig {
    pub provider: LLMProvider,
    pub api_key: SecretString,
    pub default_model: String,
    pub api_base: Option<String>,
}

impl LLMConfig {
    pub fn new_anthropic(api_key: SecretString) -> Self {
        Self {
            provider: LLMProvider::Anthropic,
            api_key,
            default_model: LLMProvider::Anthropic.default_model().to_string(),
            api_base: None,
        }
    }

    pub fn new_openai(api_key: SecretString) -> Self {
        Self {
            provider: LLMProvider::OpenAI,
            api_key,
            default_model: LLMProvider::OpenAI.default_model().to_string(),
            api_base: Some("https://api.openai.com".to_string()),
        }
    }

    pub fn new_groq(api_key: SecretString) -> Self {
        Self {
            provider: LLMProvider::Groq,
            api_key,
            default_model: LLMProvider::Groq.default_model().to_string(),
            api_base: Some("https://api.groq.com/openai".to_string()),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }
}
