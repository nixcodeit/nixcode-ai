use crate::providers::LLMProvider;
use secrecy::SecretString;

#[derive(Debug, Clone)]
pub struct LLMConfig {
    pub provider: LLMProvider,
    pub api_key: SecretString,
    pub default_model: String,
}

impl LLMConfig {
    pub fn new_anthropic() -> anyhow::Result<Self> {
        let api_key = SecretString::new(std::env::var("ANTHROPIC_API_KEY")?.into());

        Ok(Self {
            provider: LLMProvider::Anthropic,
            api_key,
            default_model: LLMProvider::Anthropic.default_model().to_string(),
        })
    }

    pub fn new_openai() -> anyhow::Result<Self> {
        let api_key = SecretString::new(std::env::var("OPENAI_API_KEY")?.into());

        Ok(Self {
            provider: LLMProvider::OpenAI,
            api_key,
            default_model: LLMProvider::OpenAI.default_model().to_string(),
        })
    }

    pub fn from_provider(provider: LLMProvider) -> anyhow::Result<Self> {
        match provider {
            LLMProvider::Anthropic => Self::new_anthropic(),
            LLMProvider::OpenAI => Self::new_openai(),
            LLMProvider::Gemini => Err(anyhow::anyhow!("Gemini API not yet supported")),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }
}
