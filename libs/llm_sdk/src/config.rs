use crate::providers::LLMProvider;
use secrecy::SecretString;

#[derive(Clone)]
pub struct HttpClientOptions {
    pub provider: LLMProvider,
    pub api_key: SecretString,
    pub api_base: Option<String>,
}

impl HttpClientOptions {
    pub fn new_anthropic(api_key: SecretString) -> Self {
        Self {
            provider: LLMProvider::Anthropic,
            api_key,
            api_base: Some("https://api.anthropic.com".to_string()),
        }
    }

    pub fn new_openai(api_key: SecretString) -> Self {
        Self {
            provider: LLMProvider::OpenAI,
            api_key,
            api_base: Some("https://api.openai.com".to_string()),
        }
    }

    pub fn new_groq(api_key: SecretString) -> Self {
        Self {
            provider: LLMProvider::Groq,
            api_key,
            api_base: Some("https://api.groq.com/openai".to_string()),
        }
    }

    pub fn new_openrouter(api_key: SecretString) -> Self {
        Self {
            provider: LLMProvider::OpenRouter,
            api_key,
            api_base: Some("https://openrouter.ai/api".to_string()),
        }
    }

    pub fn new_llama(host: String, api_key: SecretString) -> Self {
        Self {
            provider: LLMProvider::Llama,
            api_key,
            api_base: Some(host),
        }
    }

    pub fn new_genai(api_key: SecretString) -> Self {
        Self {
            provider: LLMProvider::GenAI,
            api_key,
            api_base: Some("https://generativelanguage.googleapis.com".to_string()),
        }
    }
}
