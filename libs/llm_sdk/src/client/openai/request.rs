use crate::message::common::llm_message::{LLMMessage, LLMRequest};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Convert our internal request format to OpenAI format
pub fn request_to_openai(request: &LLMRequest) -> Value {
    // First convert messages to OpenAI format
    let messages = request
        .messages
        .iter()
        .flat_map(|msg| convert_message_to_openai(msg))
        .collect::<Vec<_>>();

    // Add system message if present
    let mut all_messages = Vec::new();
    if let Some(system) = request.system.clone() {
        if !system.is_empty() {
            all_messages.push(json!({
                "role": "system",
                "content": system
            }));
        }
    }
    all_messages.extend(messages);

    // Convert tool definitions if present
    let tools = request.tools.as_ref().map(|tools| {
        tools
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
            .collect::<Vec<_>>()
    });

    // Build the final request
    let mut openai_request = json!({
        "model": request.model.model_name(),
        "messages": all_messages,
        "stream": true,
        "stream_options": {
            "include_usage": true,
        },
        "max_completion_tokens": request.max_tokens.unwrap_or(1024),
    });

    // Add temperature if present
    if let Some(temperature) = request.temperature {
        openai_request["temperature"] = json!(temperature);
    }

    // Add tools if present
    if let Some(tools) = tools {
        openai_request["tools"] = json!(tools);
        openai_request["tool_choice"] = json!("auto");
        openai_request["parallel_tool_calls"] = json!(false);
    }

    openai_request
}

/// Convert a Message to OpenAI format
pub fn convert_message_to_openai(msg: &LLMMessage) -> Vec<Value> {
    let mut texts = vec![];
    let tool_calls = msg
        .tool_calls
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|tool| {
            json!({
                "function": {
                    "name": tool.name.clone(),
                    "arguments": tool.arguments.clone(),
                },
                "type": "function",
                "id": tool.id.clone().unwrap_or_default(),
            })
        })
        .collect::<Vec<Value>>();
    let tools = msg
        .tool_results
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|result| {
            json!({
                "role": "tool",
                "content": result.result.clone(),
                "tool_call_id": result.call_id.clone().unwrap_or_default(),
            })
        })
        .collect::<Vec<Value>>();
    let role = msg.role.as_str();

    if let Some(text) = msg.text.clone() {
        texts.push(json!({
            "role": role,
            "content": text
        }));
    }

    let mut msgs = vec![];

    // Add text messages
    if !texts.is_empty() {
        msgs.extend(texts);
    }

    // Add tool calls if present
    if !tool_calls.is_empty() {
        msgs.push(json!({
            "role": role,
            "tool_calls": tool_calls,
        }));
    }

    // Add tool results if present
    if !tools.is_empty() {
        msgs.extend(tools);
    }

    msgs
}
