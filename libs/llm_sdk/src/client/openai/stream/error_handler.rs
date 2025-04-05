use crate::message::anthropic::events::{ErrorEventContent, MessageResponseStreamEvent};
use tokio::sync::mpsc::UnboundedSender;

/// Create a parsing error event
pub fn create_parsing_error(
    error_message: String,
    tx: &UnboundedSender<MessageResponseStreamEvent>,
) {
    let event = MessageResponseStreamEvent::Error {
        error: ErrorEventContent {
            message: error_message,
            r#type: "ParsingError".into(),
        },
    };
    log::debug!("{:?}", event);
    tx.send(event).ok();
}

/// Create a stream error event
pub fn create_stream_error(
    error_message: String,
    tx: &UnboundedSender<MessageResponseStreamEvent>,
) {
    let event = MessageResponseStreamEvent::Error {
        error: ErrorEventContent {
            r#type: "StreamError".into(),
            message: error_message,
        },
    };
    log::debug!("{:?}", event);
    tx.send(event).ok();
}