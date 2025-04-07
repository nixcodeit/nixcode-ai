use crate::client::request::Request;
use crate::errors::llm::LLMError;
use crate::message::common::llm_message::LLMRequest;
use serde_json::{json, Value};

/// Prepare the request body for the Anthropic API with cache control if needed
pub fn prepare_request_body(request: &LLMRequest) -> Result<Value, LLMError> {
    let request = Request::try_from(request)?;

    let mut body = serde_json::to_value(&request)
        .map_err(|e| LLMError::ParseError(format!("Failed to serialize request: {}", e)))?;

    log::debug!("Request body: {:?}", body);
    // Add cache control for ephemeral content if enabled
    apply_cache_control_to_messages(&mut body)?;
    apply_cache_control_to_system(&mut body, &request)?;
    apply_cache_control_to_tools(&mut body, &request)?;

    Ok(body)
}

/// Apply cache control to the messages in the request
fn apply_cache_control_to_messages(body: &mut Value) -> Result<(), LLMError> {
    let messages = body
        .as_object_mut()
        .ok_or_else(|| LLMError::ParseError("Invalid body structure".to_string()))?
        .get_mut("messages")
        .ok_or_else(|| LLMError::ParseError("No messages found".to_string()))?
        .as_array_mut()
        .ok_or_else(|| LLMError::ParseError("Messages not an array".to_string()))?;

    if let Some(last_message) = messages.last_mut() {
        if let Some(last_content) = last_message
            .as_object_mut()
            .and_then(|m| m.get_mut("content"))
            .and_then(|c| c.as_array_mut())
            .and_then(|arr| arr.last_mut())
        {
            if let Some(content_obj) = last_content.as_object_mut() {
                content_obj.insert("cache_control".into(), json!({"type": "ephemeral"}));
            }
        }
    }

    Ok(())
}

/// Apply cache control to the system prompt if present
fn apply_cache_control_to_system(body: &mut Value, request: &Request) -> Result<(), LLMError> {
    if request.get_system().is_some() {
        if let Some(system) = body
            .as_object_mut()
            .and_then(|b| b.get_mut("system"))
            .and_then(|s| s.as_array_mut())
            .and_then(|arr| arr.last_mut())
            .and_then(|last| last.as_object_mut())
        {
            system.insert("cache_control".into(), json!({"type": "ephemeral"}));
        }
    }

    Ok(())
}

/// Apply cache control to the tools if present
fn apply_cache_control_to_tools(body: &mut Value, request: &Request) -> Result<(), LLMError> {
    if request.get_tools().is_some() {
        if let Some(tools) = body
            .as_object_mut()
            .and_then(|b| b.get_mut("tools"))
            .and_then(|t| t.as_array_mut())
            .and_then(|arr| arr.last_mut())
            .and_then(|last| last.as_object_mut())
        {
            tools.insert("cache_control".into(), json!({"type": "ephemeral"}));
        }
    }

    Ok(())
}
