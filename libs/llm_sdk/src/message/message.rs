use crate::errors::llm::LLMError;
use crate::message::common::llm_message::LLMMessage;
use crate::message::content::Content;
use serde::{Deserialize, Serialize};

pub type Contents = Vec<Content>;

impl TryFrom<&LLMMessage> for Message {
    type Error = LLMError;

    fn try_from(value: &LLMMessage) -> Result<Self, Self::Error> {
        let mut contents = vec![];

        if let Some(text) = value.text.clone() {
            contents.push(Content::new_text(text));
        }

        if let Some(tools_calls) = value.tool_calls.clone() {
            for tool_call in tools_calls {
                contents.push(Content::ToolUse(tool_call.try_into()?));
            }
        }

        if let Some(tool_results) = value.tool_results.clone() {
            for tool_result in tool_results {
                contents.push(Content::ToolResult(tool_result.try_into()?));
            }
        }

        match value.role.as_str() {
            "user" => Ok(Message::User(contents)),
            "assistant" => Ok(Message::Assistant(contents)),
            role => Err(LLMError::Generic(format!("Invalid role: {}", role))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", content = "content")]
#[serde(rename_all = "snake_case")]
pub enum Message {
    User(Contents),
    Assistant(Contents),
}

impl Message {
    pub fn get_content(&self) -> Contents {
        match self {
            Message::User(content) => content.clone(),
            Message::Assistant(content) => content.clone(),
        }
    }

    pub fn get_content_mut(&mut self) -> &mut Contents {
        match self {
            Message::User(content) => content,
            Message::Assistant(content) => content,
        }
    }

    pub fn set_content(&mut self, new_content: Contents) {
        match self {
            Message::Assistant(content) => *content = new_content,
            Message::User(content) => *content = new_content,
        }
    }
}
