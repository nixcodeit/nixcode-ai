use crate::message::content::tools::ToolUseContent;
use crate::message::content::{Content, ContentDelta};
use crate::message::usage::Usage;
use crate::stop_reason::StopReason;
use crate::{
    ContentBlockDeltaEventContent, ContentBlockStartEventContent, MessageDeltaEventContent,
    MessageStartEventContent,
};
use serde::{Deserialize, Serialize};
use std::ops::AddAssign;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub model: String,
    pub role: String,
    pub stop_reason: Option<StopReason>,
    pub content: Vec<Content>,
    pub stop_sequence: Option<String>,
    pub usage: Usage,
}

impl MessageResponse {
    pub(crate) fn finish_content_block(&mut self, index: usize) {
        let x = self.content.get_mut(index);
        if let Some(content) = x {
            match content {
                Content::Text(text) => {
                    text.text = text.text.trim().to_string();
                }
                _ => {}
            }
        }
    }
}

impl MessageResponse {
    pub fn get_content(&self, index: usize) -> Content {
        self.content[index].clone()
    }

    pub fn is_max_tokens_output(&self) -> bool {
        self.stop_reason == Some(StopReason::MaxTokens)
    }
}

impl MessageResponse {
    pub fn tools_usage(&self) -> Vec<ToolUseContent> {
        self.content
            .clone()
            .into_iter()
            .filter(|x| match x {
                Content::ToolUse { .. } => true,
                _ => false,
            })
            .map(|x| match x {
                Content::ToolUse(tool) => tool,
                _ => unreachable!(),
            })
            .collect()
    }
}

impl MessageResponse {
    pub fn content_delta(&mut self, index: u32, delta_content: ContentDelta) -> &mut Self {
        let content = self.content.get_mut(index as usize).unwrap();
        content.extend_delta(delta_content);

        self
    }
}

impl MessageResponse {
    pub fn get_text(&self) -> String {
        let mut text2 = String::new();

        for content in &self.content {
            match content {
                Content::Text(text) => {
                    text2.push_str(text.text.as_str());
                }
                _ => {}
            }
        }

        text2
    }
}

impl AddAssign for MessageResponse {
    fn add_assign(&mut self, rhs: MessageResponse) {
        self.id = rhs.id;
        self.model = rhs.model;
        self.role = rhs.role;
        self.usage += rhs.usage;
        self.stop_reason = rhs.stop_reason;
        self.stop_sequence = rhs.stop_sequence;
        self.content.extend(rhs.content);
    }
}

impl AddAssign<MessageStartEventContent> for MessageResponse {
    fn add_assign(&mut self, rhs: MessageStartEventContent) {
        *self += rhs.message;
    }
}

impl AddAssign<ContentBlockStartEventContent> for MessageResponse {
    fn add_assign(&mut self, rhs: ContentBlockStartEventContent) {
        self.content.insert(rhs.index, rhs.content_block);
    }
}

impl AddAssign<ContentBlockDeltaEventContent> for MessageResponse {
    fn add_assign(&mut self, rhs: ContentBlockDeltaEventContent) {
        let content = self.content.get_mut(rhs.index).unwrap();
        content.extend_delta(rhs.delta);
    }
}

impl AddAssign<MessageDeltaEventContent> for MessageResponse {
    fn add_assign(&mut self, rhs: MessageDeltaEventContent) {
        self.stop_reason = rhs.delta.stop_reason;
        self.stop_sequence = rhs.delta.stop_sequence;
        self.usage.output_tokens += rhs.usage.output_tokens;
    }
}
