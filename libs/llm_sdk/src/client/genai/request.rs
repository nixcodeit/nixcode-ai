use crate::message::common::llm_message::LLMRequest;
use serde_json::{json, Value};

pub fn request_to_genai(request: &LLMRequest) -> Value {
    let messages = request
        .messages
        .iter()
        .map(|message| {
            let role = match message.role.as_str() {
                "assistant" => "model",
                role => role,
            };

            let mut parts = vec![];

            if let Some(content) = &message.text {
                parts.push(json!({
                    "text": content,
                }));
            }

            if let Some(tool_calls) = &message.tool_calls {
                for tool_call in tool_calls {
                    parts.push(json!({
                        "functionCall": {
                            "name": tool_call.name,
                            "args": serde_json::from_str::<Value>(tool_call.arguments.as_str()).unwrap_or_default(),
                        }
                    }));
                }
            }

            if let Some(tool_result) = &message.tool_results {
                for tool_result in tool_result {
                    parts.push(json!({
                        "functionResponse": {
                            "name": tool_result.name,
                            "response": {
                                "result": tool_result.result,
                            }
                        }
                    }));
                }
            }

            // if let Some(content) = &message.tool_calls

            json!({
                "role": role,
                "parts": parts,
            })
        })
        .collect::<Vec<_>>();

    let system = match &request.system {
        Some(system) => Some(json!([{ "text": system }])),
        None => None,
    };

    let tools = match &request.tools {
        Some(tools) => tools
            .iter()
            .filter_map(|tool| {
                let name = tool.name.clone();
                let description = tool.description.clone();
                let mut x = tool.input.clone();
                let val = x.as_object_mut();
                match val {
                    Some(val) => {
                        val.remove("$schema");
                        val.remove("title");

                        Some(json!({
                            "name": name,
                            "description": description,
                            "parameters": val,
                        }))
                    }
                    None => None,
                }
            })
            .collect::<Vec<_>>(),
        None => vec![],
    };

    log::debug!(
        "{}",
        serde_json::to_string_pretty(&tools).unwrap_or_default()
    );

    let mut body = json!({
        "contents": messages,
    });

    if let Some(system) = system {
        body["system_instruction"] = json!({
            "parts": system
        });
    }

    if !tools.is_empty() {
        body["tools"] = json!([{
            "functionDeclarations": tools
        }])
    }

    body
}
