use crate::client::request::Request;
use crate::client::LLMClientImpl;
use crate::config::LLMConfig;
use crate::errors::llm::LLMError;
use crate::message::anthropic::events::{ErrorEventContent, MessageResponseStreamEvent};
use crate::message::anthropic::tokens::InputTokens;
use crate::message::message::Message;
use crate::message::usage::Usage;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use secrecy::ExposeSecret;
use serde_json::json;
use std::ops::AddAssign;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

#[derive(Debug)]
pub struct AnthropicClient {
    total_usages: Usage,
    history: Vec<Message>,
    client: reqwest::Client,
}

impl AddAssign<Message> for AnthropicClient {
    fn add_assign(&mut self, rhs: Message) {
        self.history.push(rhs);
    }
}

impl AnthropicClient {
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

        let reqwest_client = reqwest::Client::builder().default_headers(headers).build();
        if reqwest_client.is_err() {
            return Err(LLMError::CreateClientError(
                "Failed to create client".to_string(),
            ));
        }

        let client = AnthropicClient {
            client: reqwest_client.unwrap(),
            history: Vec::new(),
            total_usages: Usage::default(),
        };

        Ok(client)
    }

    pub fn get_usage(&self) -> Usage {
        self.total_usages.clone()
    }

    pub fn get_messages(&self) -> Vec<Message> {
        self.history.clone()
    }
}

impl LLMClientImpl for AnthropicClient {
    async fn count_tokens(&self, request: Request) -> Result<u32, LLMError> {
        let body = serde_json::to_value(&request).unwrap();

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages/count_tokens")
            .json(&body)
            .send()
            .await;

        if response.is_err() {
            return Err(LLMError::ReqwestError);
        }

        let response = response.unwrap();

        if !response.status().is_success() {
            return Err(LLMError::InvalidResponseCode(
                response.status().as_u16(),
                response.text().await.unwrap(),
            ));
        }

        let body = response.json::<InputTokens>().await;

        if body.is_err() {
            return Err(LLMError::InvalidResponse(body.unwrap_err().to_string()));
        }

        Ok(body.unwrap().input_tokens)
    }

    async fn send(
        &self,
        request: Request,
    ) -> Result<UnboundedReceiver<MessageResponseStreamEvent>, LLMError> {
        let mut body = serde_json::to_value(&request).unwrap();
        if request.is_cache_enabled() && !request.get_messages().is_empty() {
            body.as_object_mut()
                .unwrap()
                .get_mut("messages")
                .unwrap()
                .as_array_mut()
                .unwrap()
                .last_mut()
                .unwrap()
                .as_object_mut()
                .unwrap()
                .get_mut("content")
                .unwrap()
                .as_array_mut()
                .unwrap()
                .last_mut()
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("cache_control".into(), json!({"type": "ephemeral"}));

            if request.get_system().is_some() {
                body.as_object_mut()
                    .unwrap()
                    .get_mut("system")
                    .unwrap()
                    .as_array_mut()
                    .unwrap()
                    .last_mut()
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .insert("cache_control".into(), json!({"type": "ephemeral"}));
            }
            if request.get_tools().is_some() {
                body.as_object_mut()
                    .unwrap()
                    .get_mut("tools")
                    .unwrap()
                    .as_array_mut()
                    .unwrap()
                    .last_mut()
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .insert("cache_control".into(), json!({"type": "ephemeral"}));
            }
        }

        let result = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .json(&body)
            .send()
            .await;

        if result.is_err() {
            return Err(LLMError::ReqwestError);
        }

        let response = result.unwrap();

        if !response.status().is_success() {
            return Err(LLMError::InvalidResponseCode(
                response.status().as_u16(),
                response.text().await.unwrap(),
            ));
        }

        let (tx, rx) = unbounded_channel::<MessageResponseStreamEvent>();

        tokio::spawn(async move {
            let mut stream = response.bytes_stream().eventsource();
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(event) => {
                        let event = MessageResponseStreamEvent::try_from(event);

                        if let Err(err) = event {
                            tx.send(MessageResponseStreamEvent::Error { error: err.into() })
                                .ok();
                            continue;
                        }

                        tx.send(event.unwrap()).ok();
                    }
                    Err(e) => {
                        tx.send(MessageResponseStreamEvent::Error {
                            error: ErrorEventContent {
                                r#type: "EventStreamError".into(),
                                message: e.to_string(),
                            },
                        })
                        .ok();
                    }
                };
            }
        });

        Ok(rx)
    }

    fn get_config(&self) -> LLMConfig {
        todo!()
    }
}
