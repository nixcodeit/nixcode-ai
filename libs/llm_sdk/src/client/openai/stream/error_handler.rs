use crate::errors::llm::LLMError;
use crate::message::common::llm_message::LLMEvent;
use tokio::sync::mpsc::UnboundedSender;

/// Create a parsing error event
pub fn create_parsing_error(error_message: String, tx: &UnboundedSender<LLMEvent>) {
    let event = LLMEvent::Error(LLMError::Generic(format!(
        "Parsing error: {:?}",
        error_message
    )));
    log::debug!("{:?}", event);
    tx.send(event).ok();
}

/// Create a stream error event
pub fn create_stream_error(error_message: String, tx: &UnboundedSender<LLMEvent>) {
    let msg = format!("Stream error: {}", error_message);
    let event = LLMEvent::Error(LLMError::Generic(msg));
    log::debug!("{:?}", event);
    tx.send(event).ok();
}
