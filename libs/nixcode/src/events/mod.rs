use nixcode_llm_sdk::errors::llm::LLMError;
use nixcode_llm_sdk::message::common::llm_message::{ToolCall, ToolResult};

#[derive(Debug)]
pub enum NixcodeEvent {
    GeneratingResponse,
    GeneratedResponse,
    NewMessage,
    MessageUpdated,
    Error(LLMError),
    ToolStart(ToolCall),
    ToolEnd(ToolResult),
    ToolsFinished,
}
