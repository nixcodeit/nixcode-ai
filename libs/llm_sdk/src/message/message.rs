use crate::message::content::Content;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", content = "content")]
#[serde(rename_all = "snake_case")]
pub enum Message {
    User(Vec<Content>),
    System(Vec<Content>),
    Assistant(Vec<Content>),
}

impl Message {
    pub fn get_content(&self) -> Vec<Content> {
        match self {
            Message::User(content) => content.clone(),
            Message::System(content) => content.clone(),
            Message::Assistant(content) => content.clone(),
        }
    }

    pub fn get_content_mut(&mut self) -> &mut Vec<Content> {
        match self {
            Message::User(content) => content,
            Message::System(content) => content,
            Message::Assistant(content) => content,
        }
    }

    pub fn set_content(&mut self, new_content: Vec<Content>) {
        match self {
            Message::Assistant(content) => *content = new_content,
            Message::System(content) => *content = new_content,
            Message::User(content) => *content = new_content,
        }
    }
}
