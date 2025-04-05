use crate::message::anthropic::events::{
    ContentBlockDeltaEventContent, ContentBlockStartEventContent, ContentBlockStopEventContent,
    MessageResponseStreamEvent,
};
use crate::message::content::tools::{ContentInputJsonDelta, ToolUseContent};
use crate::message::content::{Content, ContentDelta};
use crate::message::openai::events::OpenAIToolCall;
use serde_json::json;
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

/// Handle tool calls from the stream
pub fn handle_tool_calls(
    tool_calls: &[OpenAIToolCall],
    last_content_index: &mut usize,
    has_pending_end_block: &mut bool,
    tx: &UnboundedSender<MessageResponseStreamEvent>,
) {
    if tool_calls.len() > 1 {
        log::debug!("Multiple tool calls are not supported yet");
        return;
    }

    let tool_call = &tool_calls[0];
    
    // Close previous content block if moving to a new index
    if tool_call.index > *last_content_index {
        let event = MessageResponseStreamEvent::ContentBlockStop(
            ContentBlockStopEventContent { index: *last_content_index }
        );
        *has_pending_end_block = false;

        log::debug!("{:?}", event);
        tx.send(event).ok();
        *last_content_index = tool_call.index;
    }

    let tool_call_id = tool_call.id.clone().unwrap_or_default();
    let arguments = tool_call.function.arguments.clone().unwrap_or_default();

    if !tool_call_id.is_empty() {
        handle_new_tool_call(tool_call, arguments, has_pending_end_block, tx);
    } else {
        handle_tool_arguments_delta(tool_call, arguments, tx);
    }
}

/// Handle the start of a new tool call
fn handle_new_tool_call(
    tool_call: &OpenAIToolCall,
    arguments: String,
    has_pending_end_block: &mut bool,
    tx: &UnboundedSender<MessageResponseStreamEvent>,
) {
    // Parse arguments or use empty object as fallback
    let input = serde_json::from_str::<Value>(&arguments)
        .unwrap_or_else(|_| json!({}));

    // Create a tool use content
    let content = ToolUseContent {
        id: tool_call.id.clone().unwrap_or_default(),
        input,
        _input_raw: arguments,
        name: tool_call.function.name.clone().unwrap_or_default(),
        ..Default::default()
    };
    
    // Send event to start a new tool content block
    let start_content = ContentBlockStartEventContent {
        index: tool_call.index,
        content_block: Content::new_tool_use(content),
    };
    
    let event = MessageResponseStreamEvent::ContentBlockStart(start_content);
    *has_pending_end_block = true;
    
    log::debug!("{:?}", event);
    tx.send(event).ok();
}

/// Handle delta updates to tool arguments
fn handle_tool_arguments_delta(
    tool_call: &OpenAIToolCall,
    arguments: String,
    tx: &UnboundedSender<MessageResponseStreamEvent>,
) {
    // Send delta event for tool arguments
    let content = ContentBlockDeltaEventContent {
        index: tool_call.index,
        delta: ContentDelta::InputJsonDelta(ContentInputJsonDelta {
            partial_json: arguments,
        }),
    };
    
    let event = MessageResponseStreamEvent::ContentBlockDelta(content);
    log::debug!("{:?}", event);
    tx.send(event).ok();
}