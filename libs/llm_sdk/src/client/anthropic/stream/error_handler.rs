use crate::message::anthropic::events::{ErrorEventContent, MessageResponseStreamEvent};

// This function is kept for potential future use
#[allow(dead_code)]
/// Create an event stream error
pub fn create_event_stream_error(error_message: String) -> MessageResponseStreamEvent {
    MessageResponseStreamEvent::Error {
        error: ErrorEventContent {
            r#type: "EventStreamError".into(),
            message: error_message,
        },
    }
}
