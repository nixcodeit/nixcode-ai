use super::request::request_to_openai;
use super::stream::process_stream;
use crate::client::LLMClientImpl;
use crate::config::LLMConfig;
use crate::errors::llm::LLMError;
use crate::message::common::llm_message::{LLMEvent, LLMRequest};
use crate::message::message::Message;
use crate::message::usage::AnthropicUsage;
use secrecy::ExposeSecret;
use std::ops::AddAssign;
use tokio::sync::mpsc::UnboundedReceiver;

/// The OpenAI client implementation for interacting with OpenAI API
pub struct OpenAIClient {
    /// Total usage statistics for this client
    total_usages: AnthropicUsage,
    /// Message history
    history: Vec<Message>,
    /// HTTP client for API requests
    client: reqwest::Client,
    /// Client configuration
    config: LLMConfig,
}

impl AddAssign<Message> for OpenAIClient {
    fn add_assign(&mut self, rhs: Message) {
        self.history.push(rhs);
    }
}

impl OpenAIClient {
    /// Create a new OpenAI client with the given configuration
    pub fn new(options: LLMConfig) -> anyhow::Result<Self, LLMError> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", options.api_key.expose_secret())
                .parse()
                .unwrap(),
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );

        let reqwest_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|_| LLMError::CreateClientError("Failed to create client".to_string()))?;

        Ok(OpenAIClient {
            client: reqwest_client,
            history: Vec::new(),
            total_usages: AnthropicUsage::default(),
            config: options,
        })
    }

    /// Get the total usage statistics for this client
    pub fn get_usage(&self) -> AnthropicUsage {
        self.total_usages.clone()
    }

    /// Get the message history for this client
    pub fn get_messages(&self) -> Vec<Message> {
        self.history.clone()
    }
}

impl LLMClientImpl for OpenAIClient {
    async fn count_tokens(&self, request: LLMRequest) -> Result<u32, LLMError> {
        // OpenAI doesn't have a dedicated token counting endpoint like Anthropic
        // We use a simple approximation based on JSON size
        let openai_body = request_to_openai(&request);
        let request_json = serde_json::to_string(&openai_body).unwrap();

        // This is a very rough approximation (4 chars ~= 1 token)
        // For production, consider using tiktoken or a similar library
        let estimated_tokens = request_json.len() / 4;

        Ok(estimated_tokens as u32)
    }

    async fn send(&self, request: LLMRequest) -> Result<UnboundedReceiver<LLMEvent>, LLMError> {
        // Validate config
        let base_url = self
            .config
            .api_base
            .clone()
            .ok_or_else(|| LLMError::InvalidConfig("API base URL is not set".to_string()))?;

        // Convert to OpenAI format
        let openai_body = request_to_openai(&request);
        log::debug!(
            "OpenAI request body: {}",
            serde_json::to_string_pretty(&openai_body).unwrap()
        );

        // Send request
        let response = self
            .client
            .post(format!("{}/v1/chat/completions", base_url))
            .json(&openai_body)
            .send()
            .await
            .map_err(|_| LLMError::ReqwestError)?;

        // Handle error responses
        if !response.status().is_success() {
            return Err(LLMError::InvalidResponseCode(
                response.status().as_u16(),
                response.text().await.unwrap_or_default(),
            ));
        }

        // Process the stream and return the receiver
        Ok(process_stream(request.model, response).await)
    }

    fn get_config(&self) -> LLMConfig {
        self.config.clone()
    }
}
