use crate::message::anthropic::events::{
    ContentBlockDeltaEventContent, ContentBlockStartEventContent, MessageResponseStreamEvent,
};
use crate::message::content::Content;
use crate::message::content::text::ContentTextDelta;
use crate::message::content::ContentDelta;
use tokio::sync::mpsc::UnboundedSender;

/// Handle text content from the stream
pub fn handle_text_content(
    content: &str,
    has_any_content: &mut bool,
    has_pending_end_block: &mut bool,
    index: &usize,
    tx: &UnboundedSender<MessageResponseStreamEvent>,
) {
    if !*has_any_content {
        // Start a new content block for the first content
        let start_event = MessageResponseStreamEvent::ContentBlockStart(
            ContentBlockStartEventContent {
                index: 0,
                content_block: Content::new_text(content),
            }
        );

        *has_any_content = true;
        *has_pending_end_block = true;

        log::debug!("{:?}", start_event);
        tx.send(start_event).ok();
    } else {
        // Send delta for additional content
        let delta = ContentDelta::TextDelta(ContentTextDelta {
            text: content.to_string(),
        });

        let event = MessageResponseStreamEvent::ContentBlockDelta(
            ContentBlockDeltaEventContent { 
                index: *index, 
                delta 
            }
        );

        log::debug!("{:?}", event);
        tx.send(event).ok();
    }
}