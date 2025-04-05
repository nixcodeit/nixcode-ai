use crate::message::anthropic::events::MessageResponseStreamEvent;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

use super::error_handler::create_event_stream_error;

/// Process a streaming response from Anthropic
pub async fn process_stream(
    response: reqwest::Response,
) -> UnboundedReceiver<MessageResponseStreamEvent> {
    let (tx, rx) = unbounded_channel::<MessageResponseStreamEvent>();
    
    // Start a task to process the streaming response
    tokio::spawn(async move {
        let mut stream = response.bytes_stream().eventsource();
        
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(event) => {
                    // Parse event into MessageResponseStreamEvent
                    let event_result = MessageResponseStreamEvent::try_from(event);

                    match event_result {
                        Ok(event) => {
                            // Send the parsed event through the channel
                            log::debug!("{:?}", event);
                            tx.send(event).ok();
                        },
                        Err(err) => {
                            // Handle parsing error
                            tx.send(MessageResponseStreamEvent::Error { error: err.into() }).ok();
                        }
                    }
                },
                Err(e) => {
                    // Handle stream error
                    let error_event = create_event_stream_error(e.to_string());
                    log::debug!("{:?}", error_event);
                    tx.send(error_event).ok();
                }
            }
        }
    });

    rx
}