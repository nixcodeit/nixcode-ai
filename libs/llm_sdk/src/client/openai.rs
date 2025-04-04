use crate::client::request::Request;
use crate::client::LLMClientImpl;
use crate::config::LLMConfig;
use crate::errors::llm::LLMError;
use crate::message::anthropic::events::{
    ContentBlockDeltaEventContent, ContentBlockStartEventContent, ContentBlockStopEventContent,
    ErrorEventContent, MessageResponseStreamEvent,
};
use crate::message::content::tools::ToolUseContent;
use crate::message::content::{Content, ContentDelta};
use crate::message::message::Message;
use crate::message::openai::events::OpenAIStreamResponse;
use crate::message::response::MessageResponse;
use crate::message::usage::Usage;
use crate::stop_reason::StopReason;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use secrecy::ExposeSecret;
use serde_json::{json, Value};
use std::ops::AddAssign;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

#[derive(Debug)]
pub struct OpenAIClient {
    total_usages: Usage,
    history: Vec<Message>,
    client: reqwest::Client,
    config: LLMConfig,
}

impl AddAssign<Message> for OpenAIClient {
    fn add_assign(&mut self, rhs: Message) {
        self.history.push(rhs);
    }
}

impl OpenAIClient {
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

        let reqwest_client = reqwest::Client::builder().default_headers(headers).build();
        if reqwest_client.is_err() {
            return Err(LLMError::CreateClientError(
                "Failed to create client".to_string(),
            ));
        }

        let client = OpenAIClient {
            client: reqwest_client.unwrap(),
            history: Vec::new(),
            total_usages: Usage::default(),
            config: options,
        };

        Ok(client)
    }

    pub fn get_usage(&self) -> Usage {
        self.total_usages.clone()
    }

    pub fn get_messages(&self) -> Vec<Message> {
        self.history.clone()
    }

    // Convert our internal request format to OpenAI format
    fn request_to_openai(&self, request: &Request) -> Value {
        let messages = request
            .get_messages()
            .iter()
            .flat_map(|msg| {
                let role = match msg {
                    Message::User(_) => "user",
                    Message::Assistant(_) => "assistant",
                    Message::System(_) => "system",
                };

                let mut tool_use = false;
                let mut tool_calls = vec![];
                let mut tools = vec![];
                // Convert content to OpenAI format
                let content = msg
                    .get_content()
                    .iter()
                    .filter_map(|content| match content {
                        Content::Text(text) => Some(json!({ "type": "text", "text": text.text })),
                        Content::Image(img) => Some(json!({
                            "type": "image_url",
                            "image_url": {
                                "url": "image_url_would_be_here",
                                "detail": "auto"
                            }
                        })),
                        Content::ToolUse(content) => {
                            tool_use = true;
                            tool_calls.push(json!({
                                "type": "function",
                                "id": content.id,
                                "function": {
                                    "name": content.name,
                                    "arguments": serde_json::to_string(&content.input).unwrap(),
                                }
                            }));
                            None
                        }
                        Content::ToolResult(content) => {
                            tools.push(json!({
                                "role": "tool",
                                "content": content.get_content(),
                                "tool_call_id": content.get_tool_use_id(),
                            }));
                            None
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                let mut msgs = vec![];
                if content.len() > 0 {
                    msgs.push(json!({
                        "role": role,
                        "content": content,
                    }));
                }
                if tool_calls.len() > 0 {
                    msgs.push(json!({
                        "role": role,
                        "tool_calls": tool_calls,
                    }));
                }

                if tools.len() > 0 {
                    msgs.extend(tools);
                }

                msgs
            })
            .collect::<Vec<_>>();

        // Add system message if present
        let mut all_messages = Vec::new();
        if let Some(system) = request.get_system() {
            all_messages.push(json!({
                "role": "system",
                "content": system
            }));
        }
        all_messages.extend(messages);

        // Convert tool definitions if present
        let tools = if let Some(tools) = request.get_tools() {
            let openai_tools = tools
                .iter()
                .map(|tool| {
                    json!({
                        "type": "function",
                        "function": {
                            "name": tool.name,
                            "description": tool.description,
                            "parameters": tool.input
                        }
                    })
                })
                .collect::<Vec<_>>();
            Some(openai_tools)
        } else {
            None
        };

        // Build the final request
        let mut openai_request = json!({
            "model": request.get_model(),
            "messages": all_messages,
            "stream": true,
            "stream_options": {
                "include_usage": true,
            },
            // "temperature": request.get_temperature(),
            "max_completion_tokens": request.get_max_tokens(),
        });

        // // Add tools if present
        if let Some(tools) = tools {
            openai_request["tools"] = json!(tools);
            openai_request["tool_choice"] = json!("auto");
            openai_request["parallel_tool_calls"] = json!(false);
        }

        openai_request
    }
}

impl LLMClientImpl for OpenAIClient {
    async fn count_tokens(&self, request: Request) -> Result<u32, LLMError> {
        let openai_body = self.request_to_openai(&request);

        // OpenAI doesn't have a direct token counting endpoint like Anthropic,
        // so we'll just use tiktoken or a similar approach to count tokens
        // For now, we'll simulate with a simplified calculation
        let request_json = serde_json::to_string(&openai_body).unwrap();
        let estimated_tokens = request_json.len() / 4; // Very rough approximation

        Ok(estimated_tokens as u32)
    }

    async fn send(
        &self,
        request: Request,
    ) -> Result<UnboundedReceiver<MessageResponseStreamEvent>, LLMError> {
        let openai_body = self.request_to_openai(&request);

        log::debug!("Original request: {:?}", request);
        log::debug!(
            "OpenAI request body: {}",
            serde_json::to_string_pretty(&openai_body).unwrap()
        );

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .json(&openai_body)
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
                        if let Ok(val) = serde_json::from_str::<Value>(event.data.as_str()) {
                            log::debug!(
                                "SSE event value: {}",
                                serde_json::to_string_pretty(&val).unwrap()
                            );
                        } else {
                            log::debug!("SSE event: {:?}", event.data);
                        }

                        if event.data.is_empty() {
                            log::debug!("Empty event data is empty, {:?}", event);
                            continue;
                        }

                        if event.data.as_str() == "[DONE]" {
                            let event = MessageResponseStreamEvent::MessageStop;
                            log::debug!("{:?}", event);
                            tx.send(event).ok();
                            continue;
                        }

                        let stream_response =
                            serde_json::from_str::<OpenAIStreamResponse>(event.data.as_str());
                        if let Err(e) = stream_response {
                            log::debug!(
                                "====\nError parsing stream response: {:?}\n{:?}\n=====",
                                event.data,
                                e
                            );
                            tx.send(MessageResponseStreamEvent::Error {
                                error: ErrorEventContent {
                                    message: e.to_string(),
                                    r#type: "ParsingError".into(),
                                },
                            })
                            .ok();
                            continue;
                        }

                        let response = stream_response.unwrap();

                        if let Some(choice) = response.choices.first() {
                            if choice.delta.role.is_some() {
                                let message_response = MessageResponse {
                                    id: response.id,
                                    model: response.model,
                                    role: choice.delta.role.clone().unwrap_or_default(),
                                    stop_reason: None,
                                    content: vec![],
                                    stop_sequence: None,
                                    usage: Usage::default(),
                                };

                                let evt = MessageResponseStreamEvent::MessageStart(
                                    crate::message::anthropic::events::MessageStartEventContent {
                                        message: message_response,
                                    },
                                );
                                log::debug!("{:?}", evt);
                                tx.send(evt).ok();
                            }

                            // If the delta contains content, it's a content block delta
                            if let Some(content) = &choice.delta.content {
                                if !has_any_content {
                                    let start_event = MessageResponseStreamEvent::ContentBlockStart(
                                        ContentBlockStartEventContent {
                                            index: 0,
                                            content_block: Content::new_text(content),
                                        },
                                    );

                                    has_any_content = true;
                                    has_pending_end_block = true;

                                    // Return the start event - the first delta will be handled in the next call
                                    log::debug!("{:?}", start_event);
                                    tx.send(start_event).ok();
                                    continue;
                                }

                                let delta = ContentDelta::TextDelta(
                                    crate::message::content::text::ContentTextDelta {
                                        text: content.clone(),
                                    },
                                );

                                let event = MessageResponseStreamEvent::ContentBlockDelta(
                                    ContentBlockDeltaEventContent { index: 0, delta },
                                );
                                log::debug!("{:?}", event);
                                tx.send(event).ok();
                                continue;
                            }

                            if let Some(tool_calls) = &choice.delta.tool_calls {
                                if tool_calls.is_empty() {
                                    log::debug!("No tool calls found");
                                    continue;
                                }

                                if tool_calls.len() > 1 {
                                    log::debug!("Multiple tool calls found");
                                    panic!("Multiple tool calls found");
                                }

                                let tool_call = &tool_calls[0];
                                if tool_call.index > last_content_index {
                                    let evt = MessageResponseStreamEvent::ContentBlockStop(
                                        ContentBlockStopEventContent {
                                            index: last_content_index,
                                        },
                                    );
                                    has_pending_end_block = false;
                                    log::debug!("{:?}", evt);
                                    tx.send(evt).ok();

                                    last_content_index = tool_call.index;
                                }

                                if let Some(id) = &tool_call.id {
                                    let content = ToolUseContent {
                                        id: id.clone(),
                                        name: String::from(
                                            tool_call
                                                .function
                                                .name
                                                .as_ref()
                                                .unwrap_or(&"".to_string()),
                                        ),
                                        ..Default::default()
                                    };
                                    let start_content = ContentBlockStartEventContent {
                                        index: tool_call.index,
                                        content_block: Content::new_tool_use(content),
                                    };
                                    let evt = MessageResponseStreamEvent::ContentBlockStart(
                                        start_content,
                                    );
                                    has_pending_end_block = true;
                                    log::debug!("{:?}", evt);
                                    tx.send(evt).ok();
                                } else {
                                    // tool args chunks
                                    let content = ContentBlockDeltaEventContent {
                                        index: tool_call.index,
                                        delta: ContentDelta::InputJsonDelta(
                                            crate::message::content::tools::ContentInputJsonDelta {
                                                partial_json: tool_call.function.arguments.clone(),
                                            },
                                        ),
                                    };
                                    let evt =
                                        MessageResponseStreamEvent::ContentBlockDelta(content);
                                    log::debug!("{:?}", evt);
                                    tx.send(evt).ok();
                                }
                            }

                            // Create usage information if available
                            let usage = response.usage.map_or_else(
                                || crate::message::usage::UsageDelta { output_tokens: 0 },
                                |u| crate::message::usage::UsageDelta {
                                    output_tokens: u.output_tokens,
                                },
                            );

                            // Handle finish reason
                            if let Some(finish_reason) = &choice.finish_reason {
                                if has_pending_end_block {
                                    let evt = MessageResponseStreamEvent::ContentBlockStop(
                                        ContentBlockStopEventContent {
                                            index: last_content_index,
                                        },
                                    );
                                    log::debug!("{:?}", evt);
                                    tx.send(evt).ok();
                                }

                                // Map OpenAI finish reason to Anthropic stop reason
                                let stop_reason = match finish_reason.as_str() {
                                    "stop" => Some(StopReason::EndTurn),
                                    "length" => Some(StopReason::MaxTokens),
                                    "function_call" => Some(StopReason::ToolUse),
                                    "tool_calls" => Some(StopReason::ToolUse),
                                    "content_filter" => Some(StopReason::StopSequence),
                                    _ => None,
                                };

                                let event = MessageResponseStreamEvent::MessageDelta(
                                    crate::message::anthropic::events::MessageDeltaEventContent {
                                        delta: crate::MessageDelta {
                                            stop_reason,
                                            stop_sequence: None,
                                        },
                                        usage,
                                    },
                                );

                                log::debug!("{:?}", event);
                                tx.send(event).ok();
                            }
                        };

                        // let event = MessageResponseStreamEvent::try_from(stream_response);
                        // if let Err(e) = event {
                        //     tx.send(MessageResponseStreamEvent::Error {
                        //         error: e.into(),
                        //     }).ok();
                        //     continue;
                        // }
                        // let event = event.unwrap();
                        // log::debug!("{:?}", event);
                        // println!("{:?}", event);

                        // OpenAI sends "data: " prefixed chunks
                        // let chunk_str = String::from_utf8_lossy(&bytes);
                        // for line in chunk_str.lines() {
                        //     if line.starts_with("data: ") {
                        //         let data = &line[6..]; // Skip "data: "
                        //
                        //         // Handle [DONE] message
                        //         if data == "[DONE]" {
                        //             // If we had a content block, close it
                        //             if content_block_started {
                        //                 tx.send(MessageResponseStreamEvent::ContentBlockStop(
                        //                     crate::message::anthropic::events::ContentBlockStopEventContent {
                        //                         index: 0,
                        //                     },
                        //                 )).ok();
                        //             }
                        //
                        //             // Send MessageStop event
                        //             tx.send(MessageResponseStreamEvent::MessageStop).ok();
                        //             continue;
                        //         }
                        //
                        //         // Parse the JSON response
                        //         match serde_json::from_str::<OpenAIStreamResponse>(data) {
                        //             Ok(openai_response) => {
                        //                 // Convert to our internal event format
                        //                 match MessageResponseStreamEvent::try_from(openai_response) {
                        //                     Ok(event) => {
                        //                         // Update state flags based on event type
                        //                         match &event {
                        //                             MessageResponseStreamEvent::MessageStart(_) => {
                        //                                 message_started = true;
                        //                             },
                        //                             MessageResponseStreamEvent::ContentBlockStart(_) => {
                        //                                 content_block_started = true;
                        //                             },
                        //                             MessageResponseStreamEvent::MessageStop => {
                        //                                 message_started = false;
                        //                                 content_block_started = false;
                        //                             },
                        //                             MessageResponseStreamEvent::ContentBlockStop(_) => {
                        //                                 content_block_started = false;
                        //                             },
                        //                             _ => {}
                        //                         }
                        //
                        //                         // Send the event
                        //                         tx.send(event).ok();
                        //                     },
                        //                     Err(e) => {
                        //                         let content: ErrorEventContent = e.into();
                        //                         tx.send(MessageResponseStreamEvent::Error {
                        //                             error: content,
                        //                         }).ok();
                        //                     }
                        //                 }
                        //             },
                        //             Err(e) => {
                        //                 tx.send(MessageResponseStreamEvent::Error {
                        //                     error: ErrorEventContent {
                        //                         r#type: "ParsingError".into(),
                        //                         message: e.to_string(),
                        //                     },
                        //                 }).ok();
                        //             }
                        //         }
                        //     }
                        // }
                    }
                    Err(e) => {
                        tx.send(MessageResponseStreamEvent::Error {
                            error: ErrorEventContent {
                                r#type: "StreamError".into(),
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
        self.config.clone()
    }
}
