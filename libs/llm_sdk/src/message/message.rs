use crate::message::content::tools::ToolUseState;
use crate::message::content::Content;
use serde::{Deserialize, Serialize};

pub type Contents = Vec<Content>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", content = "content")]
#[serde(rename_all = "snake_case")]
pub enum Message {
    User(Contents),
    System(Contents),
    Assistant(Contents),
}

impl Message {
    pub fn get_content(&self) -> Contents {
        match self {
            Message::User(content) => content.clone(),
            Message::System(content) => content.clone(),
            Message::Assistant(content) => content.clone(),
        }
    }

    pub fn get_content_mut(&mut self) -> &mut Contents {
        match self {
            Message::User(content) => content,
            Message::System(content) => content,
            Message::Assistant(content) => content,
        }
    }

    pub fn set_content(&mut self, new_content: Contents) {
        match self {
            Message::Assistant(content) => *content = new_content,
            Message::System(content) => *content = new_content,
            Message::User(content) => *content = new_content,
        }
    }

    pub fn set_tool_state(&mut self, tool_id: String, state: ToolUseState) {
        let content = self.get_content_mut();
        for c in content {
            if let Content::ToolUse(tool_use) = c {
                if tool_use.get_id() == tool_id {
                    tool_use.set_state(state);
                    break;
                }
            }
        }
    }
}
