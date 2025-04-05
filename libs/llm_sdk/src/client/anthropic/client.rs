use crate::client::LLMClientImpl;
use crate::client::request::Request;
use crate::config::LLMConfig;
use crate::errors::llm::LLMError;
use crate::message::anthropic::events::MessageResponseStreamEvent;
use crate::message::anthropic::tokens::InputTokens;
use crate::message::message::Message;
use crate::message::usage::Usage;
use secrecy::ExposeSecret;
use std::ops::AddAssign;
use tokio::sync::mpsc::UnboundedReceiver;

use super::request::prepare_request_body;
use super::stream::process_stream;

/// The Anthropic client implementation for interacting with Anthropic API
#[derive(Debug)]
pub struct AnthropicClient {
    /// Total usage statistics for this client
    total_usages: Usage,
    /// Message history
    history: Vec<Message>,
    /// HTTP client for API requests
    client: reqwest::Client,
    /// Client configuration
    config: LLMConfig,
}

impl AddAssign<Message> for AnthropicClient {
    fn add_assign(&mut self, rhs: Message) {
        self.history.push(rhs);
    }
}

impl AnthropicClient {
    /// Create a new Anthropic client with the given configuration
    pub fn new(options: LLMConfig) -> anyhow::Result<Self, LLMError> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            options.api_key.expose_secret().parse().unwrap(),
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        headers.insert("anthropic-version", "2023-06-01".parse().unwrap());

        let reqwest_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|_| LLMError::CreateClientError("Failed to create client".to_string()))?;

        Ok(AnthropicClient {
            client: reqwest_client,
            history: Vec::new(),
            total_usages: Usage::default(),
            config: options,
        })
    }

    /// Get the total usage statistics for this client
    pub fn get_usage(&self) -> Usage {
        self.total_usages.clone()
    }

    /// Get the message history for this client
    pub fn get_messages(&self) -> Vec<Message> {
        self.history.clone()
    }
}

impl LLMClientImpl for AnthropicClient {
    async fn count_tokens(&self, request: Request) -> Result<u32, LLMError> {
        let body = serde_json::to_value(&request)
            .map_err(|e| LLMError::ParseError(format!("Failed to serialize request: {}", e)))?;

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages/count_tokens")
            .json(&body)
            .send()
            .await
            .map_err(|_| LLMError::ReqwestError)?;

        if !response.status().is_success() {
            return Err(LLMError::InvalidResponseCode(
                response.status().as_u16(),
                response.text().await.unwrap_or_default(),
            ));
        }

        let body = response.json::<InputTokens>().await
            .map_err(|e| LLMError::InvalidResponse(e.to_string()))?;

        Ok(body.input_tokens)
    }

    async fn send(
        &self,
        request: Request,
    ) -> Result<UnboundedReceiver<MessageResponseStreamEvent>, LLMError> {
        // Prepare request body with cache control if needed
        let body = prepare_request_body(&request)?;

        // Send the request to Anthropic API
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .json(&body)
            .send()
            .await
            .map_err(|_| LLMError::ReqwestError)?;

        // Check for unsuccessful response
        if !response.status().is_success() {
            return Err(LLMError::InvalidResponseCode(
                response.status().as_u16(),
                response.text().await.unwrap_or_default(),
            ));
        }

        // Process the response stream
        Ok(process_stream(response).await)
    }

    fn get_config(&self) -> LLMConfig {
        self.config.clone()
    }
}