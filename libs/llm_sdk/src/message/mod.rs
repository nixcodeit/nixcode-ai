pub mod anthropic;
pub mod common;
pub mod content;
pub mod genai;
pub mod message;
pub mod openai;
pub mod response;
pub mod usage;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MessageDelta {
    pub stop_reason: Option<crate::stop_reason::StopReason>,
    pub stop_sequence: Option<String>,
}
