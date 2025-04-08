use super::request::prepare_request_body;
use super::stream::process_stream;
use crate::client::request::Request;
use crate::client::LLMClientImpl;
use crate::config::HttpClientOptions;
use crate::errors::llm::LLMError;
use crate::message::anthropic::tokens::InputTokens;
use crate::message::common::llm_message::{LLMEvent, LLMRequest};
use crate::message::message::Message;
use crate::message::usage::AnthropicUsage;
use secrecy::ExposeSecret;
use std::ops::AddAssign;
use tokio::sync::mpsc::UnboundedReceiver;

/// The Anthropic client implementation for interacting with Anthropic API
pub struct AnthropicClient {
    /// Total usage statistics for this client
    total_usages: AnthropicUsage,
    /// Message history
    history: Vec<Message>,
    /// HTTP client for API requests
    client: reqwest::Client,
    /// Client configuration
    config: HttpClientOptions,
}

impl AddAssign<Message> for AnthropicClient {
    fn add_assign(&mut self, rhs: Message) {
        self.history.push(rhs);
    }
}

impl AnthropicClient {
    /// Create a new Anthropic client with the given configuration
    pub fn new(options: HttpClientOptions) -> anyhow::Result<Self, LLMError> {
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

impl LLMClientImpl for AnthropicClient {
    async fn count_tokens(&self, request: LLMRequest) -> Result<u32, LLMError> {
        let request = Request::try_from(&request)?;
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

        let body = response
            .json::<InputTokens>()
            .await
            .map_err(|e| LLMError::InvalidResponse(e.to_string()))?;

        Ok(body.input_tokens)
    }

    async fn send(&self, request: LLMRequest) -> Result<UnboundedReceiver<LLMEvent>, LLMError> {
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
        Ok(process_stream(request.model, response).await)
    }

    fn get_config(&self) -> HttpClientOptions {
        self.config.clone()
    }
}
