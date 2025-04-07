pub mod client;
pub mod config;
pub mod errors;
pub mod json_schema;
pub mod message;
pub mod models;
pub mod providers;
pub mod stop_reason;
pub mod tools;

use futures::StreamExt;
use message::content::Content;
pub use message::MessageDelta;
use serde::{Deserialize, Serialize};
use stop_reason::StopReason;

#[derive(Debug)]
pub struct Response {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThinkingOptions {
    r#type: String,
    budget_tokens: u32,
}

impl ThinkingOptions {
    pub fn new(budget_tokens: u32) -> Self {
        Self {
            r#type: "enabled".into(),
            budget_tokens,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum LLMEvent {
    PartialContent(usize, Content),
    Content(Content),
    Stop(Option<StopReason>),
}
