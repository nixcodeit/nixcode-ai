use crate::client::request::Request;
use crate::message::content::Content;
use crate::message::message::Message;
use serde_json::{json, Value};

/// Convert our internal request format to OpenAI format
pub fn request_to_openai(request: &Request) -> Value {
    // First convert messages to OpenAI format
    let messages = request
        .get_messages()
        .iter()
        .flat_map(|msg| convert_message_to_openai(msg))
        .collect::<Vec<_>>();

    // Add system message if present
    let mut all_messages = Vec::new();
    if let Some(system) = request.get_system() {
        let system_message = system
            .iter()
            .flat_map(|x| x.get_text())
            .map(|x| x.text.clone())
            .collect::<Vec<_>>()
            .join("\n\n");

        if !system_message.is_empty() {
            all_messages.push(json!({
                "role": "system",
                "content": system_message
            }));
        }
    }
    all_messages.extend(messages);

    // Convert tool definitions if present
    let tools = request.get_tools().as_ref().map(|tools| {
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
        "model": request.get_model(),
        "messages": all_messages,
        "stream": true,
        "stream_options": {
            "include_usage": true,
        },
        "max_completion_tokens": request.get_max_tokens(),
    });

    // Add temperature if present
    if let Some(temperature) = request.get_temperature() {
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
pub fn convert_message_to_openai(msg: &Message) -> Vec<Value> {
    let role = match msg {
        Message::User(_) => "user",
        Message::Assistant(_) => "assistant",
        Message::System(_) => "system",
    };

    let mut texts = vec![];
    let mut tool_calls = vec![];
    let mut tools = vec![];

    // Process each content item
    for content in msg.get_content() {
        match content {
            Content::Text(text) => {
                texts.push(json!({
                    "role": role,
                    "content": text.text
                }));
            },
            Content::ToolUse(content) => {
                tool_calls.push(json!({
                    "type": "function",
                    "id": content.id,
                    "function": {
                        "name": content.name,
                        "arguments": serde_json::to_string(&content.input).unwrap(),
                    }
                }));
            },
            Content::ToolResult(content) => {
                tools.push(json!({
                    "role": "tool",
                    "content": content.get_content(),
                    "tool_call_id": content.get_tool_use_id(),
                }));
            },
            _ => (),
        }
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