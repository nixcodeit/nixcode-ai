use crate::client::openai::stream::error_handler::{create_parsing_error, create_stream_error};
use crate::message::anthropic::events::{
    ContentBlockStopEventContent, MessageDeltaEventContent, MessageResponseStreamEvent,
    MessageStartEventContent,
};
use crate::message::common::llm_message::{LLMEvent, LLMMessage, Usage};
use crate::message::openai::events::OpenAIStreamResponse;
use crate::message::openai::{OpenAIChoice, OpenAIMessageToolCall, OpenAIResponse};
use crate::message::response::MessageResponse;
use crate::message::usage::UsageDelta;
use crate::models::llm_model::LLMModel;
use crate::stop_reason::StopReason;
use crate::MessageDelta;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use serde_json::Value;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

/// Process a streaming response from OpenAI
pub async fn process_stream(
    model: &'static LLMModel,
    response: reqwest::Response,
) -> UnboundedReceiver<LLMEvent> {
    let (tx, rx) = unbounded_channel::<LLMEvent>();

    // Start a task to process the streaming response
    tokio::spawn(async move {
        let mut stream = response.bytes_stream().eventsource();
        let mut response = OpenAIResponse::default();

        let send_update = |response: OpenAIResponse| {
            log::debug!("Response: {:?}", response);
            let mut usage = Usage::from(response.usage.clone().unwrap_or_default());
            let usage_cost = model.calculate_cost(usage.clone());
            usage.cost = usage_cost;

            let msg = LLMMessage::from(response).with_usage(usage).to_owned();

            let event = LLMEvent::MessageUpdate(msg);
            log::debug!("{:?}", event);
            tx.send(event).ok();
        };

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(event) => {
                    // Log the event data for debugging
                    if let Ok(val) = serde_json::from_str::<Value>(event.data.as_str()) {
                        log::debug!(
                            "SSE event value: {}",
                            serde_json::to_string_pretty(&val).unwrap()
                        );
                    } else {
                        log::debug!("SSE event: {:?}", event.data);
                    }

                    // Skip empty events
                    if event.data.is_empty() {
                        log::debug!("Empty event data, {:?}", event);
                        continue;
                    }

                    if event.data.as_str() == "[DONE]" {
                        let event = LLMEvent::MessageComplete;
                        log::debug!("{:?}", event);
                        tx.send(event).ok();
                        continue;
                    }

                    let stream_response =
                        match serde_json::from_str::<OpenAIStreamResponse>(event.data.as_str()) {
                            Ok(response) => response,
                            Err(e) => {
                                create_parsing_error(e.to_string(), &tx);
                                break;
                            }
                        };

                    log::debug!("OpenAIStreamResponse: {:?}", stream_response);

                    match (response.usage.as_mut(), stream_response.usage) {
                        (Some(current_usage), Some(new_usage)) => {
                            current_usage.completion_tokens += &new_usage.completion_tokens;
                            current_usage.prompt_tokens += &new_usage.prompt_tokens;
                            current_usage.total_tokens += &new_usage.total_tokens;
                        }
                        (None, Some(new_usage)) => {
                            response.usage = Some(new_usage.clone());
                        }
                        (_, _) => (),
                    }

                    let choice = match stream_response.choices.first() {
                        Some(choice) => choice,
                        None => {
                            send_update(response.clone());
                            continue;
                        }
                    };

                    if let Some(_) = &choice.delta.role {
                        response.set_id(stream_response.id.clone());
                        response.model = stream_response.model.clone();
                    }

                    let content = match &choice.delta.content {
                        Some(content) => content.clone(),
                        None => "".into(),
                    };

                    let response_choice = match response.choices.get_mut(choice.index) {
                        Some(choice) => choice,
                        None => {
                            response.choices.push(OpenAIChoice::default());
                            response.choices.last_mut().unwrap()
                        }
                    };

                    if !content.is_empty() {
                        response_choice.message.content += &content;
                    }

                    let tools = match &choice.delta.tool_calls {
                        Some(tool_calls) => tool_calls.clone(),
                        None => vec![],
                    };

                    for tool in tools {
                        let tool_call = match response_choice.message.tool_calls.get_mut(tool.index)
                        {
                            Some(tool_call) => tool_call,
                            None => {
                                response_choice
                                    .message
                                    .tool_calls
                                    .push(OpenAIMessageToolCall::default());
                                response_choice
                                    .message
                                    .tool_calls
                                    .get_mut(tool.index)
                                    .unwrap()
                            }
                        };

                        let id = tool.id.unwrap_or_default();
                        if !id.is_empty() {
                            tool_call.id = id;
                        }

                        let name = tool.function.name.unwrap_or_default();
                        if !name.is_empty() {
                            tool_call.function.name = name;
                        }

                        let arguments = tool.function.arguments.unwrap_or_default();
                        if !arguments.is_empty() {
                            tool_call.function.arguments += &arguments;
                        }
                    }

                    if let Some(stop_reason) = &choice.finish_reason {
                        response_choice.finish_reason = Some(stop_reason.clone());
                    }

                    send_update(response.clone());
                }
                Err(e) => {
                    create_stream_error(e.to_string(), &tx);
                }
            }
        }
    });

    rx
}
