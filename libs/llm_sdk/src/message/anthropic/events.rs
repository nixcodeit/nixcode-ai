use crate::errors::llm::LLMError;
use crate::message::content::{Content, ContentDelta};
use crate::message::response::MessageResponse;
use crate::message::usage::UsageDelta;
use crate::MessageDelta;
use eventsource_stream::Event;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedReceiver;

pub type MessageResponseStream = UnboundedReceiver<MessageResponseStreamEvent>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MessageStartEventContent {
    pub message: MessageResponse,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ContentBlockStartEventContent {
    pub index: usize,
    pub content_block: Content,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ContentBlockDeltaEventContent {
    pub index: usize,
    pub delta: ContentDelta,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ContentBlockStopEventContent {
    pub index: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MessageDeltaEventContent {
    pub delta: MessageDelta,
    pub usage: UsageDelta,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorEventContent {
    pub(crate) r#type: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum MessageResponseStreamEvent {
    MessageStart(MessageStartEventContent),
    ContentBlockStart(ContentBlockStartEventContent),
    ContentBlockDelta(ContentBlockDeltaEventContent),
    ContentBlockStop(ContentBlockStopEventContent),
    MessageDelta(MessageDeltaEventContent),
    MessageStop,
    Ping,
    Error { error: ErrorEventContent },
}

impl TryFrom<Event> for MessageResponseStreamEvent {
    type Error = LLMError;

    fn try_from(value: Event) -> Result<Self, LLMError> {
        let data = value.data;

        let event = serde_json::from_str::<MessageResponseStreamEvent>(&data);

        if let Err(err) = event {
            return Err(LLMError::ParseError(err.to_string()));
        }

        Ok(event.unwrap())
    }
}
