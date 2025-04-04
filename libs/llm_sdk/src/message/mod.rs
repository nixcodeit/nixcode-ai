pub mod content;
pub mod message;
pub mod response;
pub mod usage;
pub mod anthropic;
pub mod openai;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MessageDelta {
    pub stop_reason: Option<crate::stop_reason::StopReason>,
    pub stop_sequence: Option<String>,
}