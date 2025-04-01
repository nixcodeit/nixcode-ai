use nixcode_llm_sdk::errors::llm::LLMError;
use nixcode_llm_sdk::message::content::tools::{ToolResultContent, ToolUseContent};

pub enum NixcodeEvent {
    GeneratingResponse,
    GeneratedResponse,
    NewMessage,
    MessageUpdated,
    Error(LLMError),
    ToolStart(ToolUseContent),
    ToolEnd(ToolResultContent),
    ToolsFinished,
}
