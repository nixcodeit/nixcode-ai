use crate::message::content::image::ImageContent;
use crate::message::content::text::{ContentTextDelta, TextContent};
use crate::message::content::thinking::{
    ContentSignatureDelta, ContentThinkingDelta, RedactedThinkingContent, ThinkingContent,
};
use crate::message::content::tools::{ContentInputJsonDelta, ToolResultContent, ToolUseContent};
use serde::{Deserialize, Serialize};

pub mod image;
pub mod image_source;
pub mod text;
pub mod thinking;
pub mod tools;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Content {
    Empty,
    Text(TextContent),
    Image(ImageContent),
    Thinking(ThinkingContent),
    RedactedThinking(RedactedThinkingContent),
    ToolUse(ToolUseContent),
    ToolResult(ToolResultContent),
}

impl Content {
    pub fn is_text(&self) -> bool {
        matches!(self, Content::Text(_))
    }

    pub fn get_text(&self) -> Option<TextContent> {
        match self {
            Content::Text(text) => Some(text.clone()),
            _ => None,
        }
    }

    pub fn extend_text(&mut self, new_text: impl Into<String>) {
        match self {
            Content::Text(text) => {
                text.text = format!("{}{}", text.text, new_text.into());
            }
            _ => {}
        }
    }

    pub fn is_tool_use(&self) -> bool {
        matches!(self, Content::ToolUse(_))
    }

    pub(crate) fn set_tool_state(&mut self, state: tools::ToolUseState) {
        if let Content::ToolUse(tool_use) = self {
            tool_use.set_state(state);
        }
    }

    pub fn validate_content(&self) -> bool {
        match self {
            Content::Text(text) => text.validate_content(),
            Content::Thinking(thinking) => thinking.validate_content(),
            Content::ToolUse(tool_use) => tool_use.validate_content(),
            Content::ToolResult(tool_result) => tool_result.validate_content(),
            _ => true,
        }
    }
}

impl Content {
    pub fn new_text(text: impl Into<String>) -> Self {
        Content::Text(TextContent::new(text.into()))
    }

    pub fn new_tool_result(result: ToolResultContent) -> Self {
        Content::ToolResult(result)
    }

    pub fn new_tool_use(tool_use: ToolUseContent) -> Self {
        Content::ToolUse(tool_use)
    }

    pub fn new_tool_results(results: Vec<ToolResultContent>) -> Vec<Content> {
        results.into_iter().map(Content::new_tool_result).collect()
    }
}

impl Into<Vec<Content>> for Content {
    fn into(self) -> Vec<Content> {
        vec![self]
    }
}

impl Content {
    pub fn extend_delta(&mut self, delta: ContentDelta) {
        match (self, delta) {
            (Content::Text(text), ContentDelta::TextDelta(delta)) => {
                *text += delta;
            }
            (Content::Thinking(thinking), ContentDelta::ThinkingDelta(delta)) => {
                *thinking += delta;
            }
            (Content::Thinking(thinking), ContentDelta::SignatureDelta(delta)) => {
                *thinking += delta;
            }
            (Content::ToolUse(tools), ContentDelta::InputJsonDelta(delta)) => {
                *tools += delta;
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ContentDelta {
    TextDelta(ContentTextDelta),
    ThinkingDelta(ContentThinkingDelta),
    SignatureDelta(ContentSignatureDelta),
    InputJsonDelta(ContentInputJsonDelta),
}
