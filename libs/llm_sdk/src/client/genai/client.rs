use crate::client::genai::request::request_to_genai;
use crate::client::LLMClientImpl;
use crate::config::HttpClientOptions;
use crate::errors::llm::LLMError;
use crate::message::common::llm_message::{LLMEvent, LLMMessage, LLMRequest, ToolCall, Usage};
use crate::message::genai::GenAIEvent;
use crate::stop_reason::StopReason;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use log::error;
use secrecy::ExposeSecret;
use std::future::Future;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

pub struct GenAIClient {
    pub client: reqwest::Client,
    pub config: HttpClientOptions,
}

impl GenAIClient {
    pub fn new(config: HttpClientOptions) -> anyhow::Result<GenAIClient, LLMError> {
        Ok(GenAIClient {
            client: reqwest::Client::builder()
                .build()
                .map_err(|_| LLMError::CreateClientError("Failed to create client".to_string()))?,
            config,
        })
    }
}

impl LLMClientImpl for GenAIClient {
    async fn count_tokens(&self, _: LLMRequest) -> Result<u32, LLMError> {
        Ok(0)
    }

    async fn send(&self, request: LLMRequest) -> Result<UnboundedReceiver<LLMEvent>, LLMError> {
        let body = request_to_genai(&request);

        let url = format!(
            "{}/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
            self.config.api_base.clone().unwrap_or_default(),
            request.model.model_name(),
            self.config.api_key.expose_secret().to_string()
        );

        log::debug!("{}", body);
        let response = self
            .client
            .post(url)
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

        let (tx, rx) = unbounded_channel::<LLMEvent>();

        tokio::spawn(async move {
            let mut stream = response.bytes_stream().eventsource();

            let mut message = LLMMessage::assistant();

            tx.send(LLMEvent::MessageStart).ok();
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(event) => {
                        log::debug!("[RAW]: {:?}", event.data);
                        let evt = match serde_json::from_str::<GenAIEvent>(event.data.as_str()) {
                            Ok(evt) => evt,
                            Err(e) => {
                                log::error!("Failed to parse event: {}", e);
                                continue;
                            }
                        };

                        let candidate = match evt.candidates.first() {
                            Some(candidate) => candidate,
                            None => {
                                continue;
                            }
                        };

                        if let Some(reason) = &candidate.finish_reason {
                            match reason.as_str() {
                                "STOP" => {
                                    message.stop_reason = Some(StopReason::EndTurn);
                                }
                                reason => log::debug!("Unknown finish reason: {}", reason),
                            }
                        }

                        if let Some(x) = evt.usage_metadata {
                            let usage = Usage::from(x);
                            message.add_usage(usage);
                        }

                        let content = match candidate.content.parts.first() {
                            Some(content) => content,
                            None => {
                                continue;
                            }
                        };

                        if let Some(text) = &content.text {
                            if !text.is_empty() {
                                message.add_text(text);
                            }
                        }

                        if let Some(tool_call) = &content.function_call {
                            let call = ToolCall::from(tool_call);
                            message.add_tool_call(call);
                        }

                        log::debug!("{:?}", message);

                        if let Some(usage) = &mut message.usage {
                            usage.cost = request.model.calculate_cost(usage.clone());
                        }

                        tx.send(LLMEvent::MessageUpdate(message.clone())).ok();
                    }
                    Err(e) => {
                        log::error!("ERROR: {}", e);
                    }
                }
            }
            tx.send(LLMEvent::MessageComplete).ok();
            log::debug!("Exiting generator loop");
        });

        Ok(rx)
    }

    fn get_config(&self) -> HttpClientOptions {
        self.config.clone()
    }
}
