use crate::config::LLMConfig;
use crate::errors::llm::LLMError;
use crate::message::common::llm_message::{LLMEvent, LLMRequest};
use anthropic::AnthropicClient;
use openai::OpenAIClient;
use tokio::sync::mpsc::UnboundedReceiver;

pub mod anthropic;
pub mod openai;
pub mod request;

pub enum LLMClient {
    OpenAI(OpenAIClient),
    Anthropic(AnthropicClient),
}

impl LLMClient {
    pub fn new_openai(options: LLMConfig) -> anyhow::Result<Self, LLMError> {
        let client = OpenAIClient::new(options);

        if let Err(client) = client {
            return Err(client);
        }

        Ok(LLMClient::OpenAI(client?))
    }

    pub fn new_anthropic(options: LLMConfig) -> anyhow::Result<Self, LLMError> {
        let client = AnthropicClient::new(options);

        if let Err(client) = client {
            return Err(client);
        }

        Ok(LLMClient::Anthropic(client?))
    }

    pub async fn count_tokens(&self, request: LLMRequest) -> Result<u32, LLMError> {
        match self {
            LLMClient::OpenAI(client) => client.count_tokens(request).await,
            LLMClient::Anthropic(client) => client.count_tokens(request).await,
        }
    }

    pub async fn send(&self, request: LLMRequest) -> Result<UnboundedReceiver<LLMEvent>, LLMError> {
        match self {
            LLMClient::OpenAI(client) => client.send(request).await,
            LLMClient::Anthropic(client) => client.send(request).await,
        }
    }
}

pub trait LLMClientImpl {
    fn count_tokens(
        &self,
        request: LLMRequest,
    ) -> impl std::future::Future<Output = Result<u32, LLMError>> + Sync;
    fn send(
        &self,
        request: LLMRequest,
    ) -> impl std::future::Future<Output = Result<UnboundedReceiver<LLMEvent>, LLMError>> + Sync;

    fn get_config(&self) -> LLMConfig;
}
