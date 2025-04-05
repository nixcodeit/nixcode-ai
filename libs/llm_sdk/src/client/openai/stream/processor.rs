use crate::message::anthropic::events::{
    ContentBlockStopEventContent, MessageDeltaEventContent, MessageResponseStreamEvent,
    MessageStartEventContent,
};
use crate::message::openai::events::OpenAIStreamResponse;
use crate::message::response::MessageResponse;
use crate::message::usage::{Usage, UsageDelta};
use crate::MessageDelta;
use crate::stop_reason::StopReason;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use serde_json::Value;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

use super::error_handler::{create_parsing_error, create_stream_error};
use super::text_handler::handle_text_content;
use super::tool_handler::handle_tool_calls;

/// Process a streaming response from OpenAI
pub async fn process_stream(
    response: reqwest::Response,
) -> UnboundedReceiver<MessageResponseStreamEvent> {
    let (tx, rx) = unbounded_channel::<MessageResponseStreamEvent>();
    
    // Start a task to process the streaming response
    tokio::spawn(async move {
        let mut stream = response.bytes_stream().eventsource();
        let mut last_content_index = 0;
        let mut has_any_content = false;
        let mut has_pending_end_block = false;

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(event) => {
                    // Log the event data for debugging
                    if let Ok(val) = serde_json::from_str::<Value>(event.data.as_str()) {
                        log::debug!("SSE event value: {}", serde_json::to_string_pretty(&val).unwrap());
                    } else {
                        log::debug!("SSE event: {:?}", event.data);
                    }

                    // Skip empty events
                    if event.data.is_empty() {
                        log::debug!("Empty event data, {:?}", event);
                        continue;
                    }

                    // Handle [DONE] marker
                    if event.data.as_str() == "[DONE]" {
                        let event = MessageResponseStreamEvent::MessageStop;
                        log::debug!("{:?}", event);
                        tx.send(event).ok();
                        continue;
                    }

                    // Parse the stream response
                    let stream_response = match serde_json::from_str::<OpenAIStreamResponse>(event.data.as_str()) {
                        Ok(response) => response,
                        Err(e) => {
                            create_parsing_error(e.to_string(), &tx);
                            break;
                        }
                    };

                    process_stream_response(
                        &stream_response, 
                        &tx, 
                        &mut last_content_index,
                        &mut has_any_content,
                        &mut has_pending_end_block
                    ).await;
                }
                Err(e) => {
                    create_stream_error(e.to_string(), &tx);
                }
            }
        }
    });

    rx
}

/// Process a single stream response from OpenAI
async fn process_stream_response(
    stream_response: &OpenAIStreamResponse,
    tx: &tokio::sync::mpsc::UnboundedSender<MessageResponseStreamEvent>,
    last_content_index: &mut usize,
    has_any_content: &mut bool,
    has_pending_end_block: &mut bool,
) {
    // Process the first choice (we don't support multiple choices yet)
    if let Some(choice) = stream_response.choices.first() {
        // Handle role information at the start of a message
        if let Some(role) = &choice.delta.role {
            let message_response = MessageResponse {
                id: stream_response.id.clone(),
                model: stream_response.model.clone(),
                role: role.clone(),
                stop_reason: None,
                content: vec![],
                stop_sequence: None,
                usage: Usage::default(),
            };

            let event = MessageResponseStreamEvent::MessageStart(
                MessageStartEventContent { message: message_response }
            );
            log::debug!("{:?}", event);
            tx.send(event).ok();
        }

        // Handle text content
        if let Some(content) = &choice.delta.content {
            if !content.is_empty() {
                handle_text_content(
                    content, 
                    has_any_content, 
                    has_pending_end_block,
                    &choice.index, 
                    tx
                );
            }
        }

        // Handle tool calls
        if let Some(tool_calls) = &choice.delta.tool_calls {
            if !tool_calls.is_empty() {
                handle_tool_calls(
                    tool_calls,
                    last_content_index,
                    has_pending_end_block,
                    tx
                );
            }
        }

        // Handle finish reason and usage
        handle_completion(
            stream_response,
            choice, 
            has_pending_end_block, 
            last_content_index, 
            tx
        );
    }
}

/// Handle completion of a message or a content block
fn handle_completion(
    stream_response: &OpenAIStreamResponse,
    choice: &crate::message::openai::events::OpenAIStreamChoice,
    has_pending_end_block: &mut bool,
    last_content_index: &usize,
    tx: &tokio::sync::mpsc::UnboundedSender<MessageResponseStreamEvent>,
) {
    // Create usage information if available
    let usage = stream_response.usage.as_ref().map_or_else(
        || UsageDelta { output_tokens: 0 },
        |u| UsageDelta { output_tokens: u.output_tokens }
    );

    // Handle finish reason
    if let Some(finish_reason) = &choice.finish_reason {
        if *has_pending_end_block {
            let event = MessageResponseStreamEvent::ContentBlockStop(
                ContentBlockStopEventContent { index: *last_content_index }
            );
            log::debug!("{:?}", event);
            tx.send(event).ok();
        }

        // Map OpenAI finish reason to our stop reason format
        let stop_reason = match finish_reason.as_str() {
            "stop" => Some(StopReason::EndTurn),
            "length" => Some(StopReason::MaxTokens),
            "function_call" | "tool_calls" => Some(StopReason::ToolUse),
            "content_filter" => Some(StopReason::StopSequence),
            _ => None,
        };

        let event = MessageResponseStreamEvent::MessageDelta(
            MessageDeltaEventContent {
                delta: MessageDelta {
                    stop_reason,
                    stop_sequence: None,
                },
                usage,
            }
        );
        log::debug!("{:?}", event);
        tx.send(event).ok();
    }
}