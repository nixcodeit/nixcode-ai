use secrecy::SecretString;

#[derive(Debug, Clone)]
pub struct LLMConfig {
    pub api_key: SecretString,
}

impl LLMConfig {
    pub fn new_anthropic() -> anyhow::Result<Self> {
        let api_key = SecretString::new(std::env::var("ANTHROPIC_API_KEY")?.into());

        Ok(Self { api_key })
    }
}