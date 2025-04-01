use nixcode_llm_sdk::errors::llm::LLMError;
use nixcode_llm_sdk::message::content::tools::{ToolResultContent, ToolUseContent};
use nixcode_llm_sdk::MessageResponseStreamEvent;

pub enum NixcodeEvent {
    GeneratingResponse,
    GeneratedResponse,
    NewMessage,
    Error(LLMError),
    ToolStart(ToolUseContent),
    ToolEnd(ToolResultContent),
    ToolsFinished,
    MessageChunk(MessageResponseStreamEvent),
    ToolAddToExecute(ToolUseContent),
}
