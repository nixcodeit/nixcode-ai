use crate::errors::llm::LLMError;
use crate::message::anthropic::events::MessageResponseStreamEvent;
use crate::message::common::llm_message::{LLMEvent, LLMMessage, Usage};
use crate::message::content::Content;
use crate::message::response::MessageResponse;
use crate::message::usage::AnthropicUsage;
use crate::models::llm_model::LLMModel;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

/// Process a streaming response from Anthropic
pub async fn process_stream(
    model: Arc<LLMModel>,
    response: reqwest::Response,
) -> UnboundedReceiver<LLMEvent> {
    let (tx, rx) = unbounded_channel::<LLMEvent>();

    tokio::spawn(async move {
        let mut stream = response.bytes_stream().eventsource();

        let mut last_response = MessageResponse::default();
        let mut usage = AnthropicUsage::default();
        while let Some(chunk) = stream.next().await {
            let mut message_updated = false;
            match chunk {
                Ok(event) => {
                    let event_result = MessageResponseStreamEvent::try_from(event);

                    match event_result {
                        Ok(event) => {
                            match event {
                                MessageResponseStreamEvent::MessageStart(msg) => {
                                    last_response += msg.clone();
                                    usage += last_response.usage.clone();
                                    message_updated = true;
                                }
                                MessageResponseStreamEvent::MessageDelta(delta) => {
                                    usage.output_tokens += delta.usage.output_tokens;
                                    last_response += delta.clone();
                                    message_updated = true;
                                    log::debug!("MessageDelta: {:?}", last_response);
                                }
                                MessageResponseStreamEvent::ContentBlockStart(content) => {
                                    last_response += content.clone();
                                    message_updated = true;
                                    log::debug!("ContentBlockStart: {:?}", last_response);
                                }
                                MessageResponseStreamEvent::ContentBlockDelta(delta) => {
                                    let index = delta.index;
                                    last_response += delta.clone();
                                    log::debug!("ContentBlockDelta: {:?}", last_response);

                                    match last_response.get_content(index) {
                                        Content::ToolUse(_) => (),
                                        _ => {
                                            message_updated = true;
                                        }
                                    }
                                }
                                MessageResponseStreamEvent::ContentBlockStop(_) => {}
                                MessageResponseStreamEvent::Error { error } => {
                                    log::error!("Error: {:?}", error);
                                    tx.send(LLMEvent::Error(LLMError::Generic(error.message)))
                                        .ok();
                                    break;
                                }
                                _ => (),
                            }

                            if message_updated {
                                let mut usage = Usage::from(usage.clone());
                                let cost = model.calculate_cost(usage.clone());
                                usage.cost = cost;

                                let message = LLMMessage::from(&last_response)
                                    .with_usage(usage.clone())
                                    .to_owned();

                                tx.send(LLMEvent::MessageUpdate(message.clone())).ok();
                            }
                        }
                        Err(err) => {
                            tx.send(LLMEvent::Error(err)).ok();
                            break;
                        }
                    }
                }
                Err(e) => {
                    let event = LLMEvent::Error(LLMError::InvalidResponse(e.to_string()));
                    log::debug!("StreamError: {:?}", event);
                    tx.send(event).ok();
                    break;
                }
            }
        }

        tx.send(LLMEvent::MessageComplete).ok();
    });

    rx
}
