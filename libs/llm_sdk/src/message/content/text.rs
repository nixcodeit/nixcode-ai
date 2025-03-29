use serde::{Deserialize, Serialize};
use std::ops::AddAssign;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TextContent {
    pub text: String,
}

impl TextContent {
    pub fn new(text: String) -> Self {
        Self {
            text: text.trim().to_string(),
        }
    }

    pub fn get_text(&self) -> String {
        self.text.clone()
    }

    pub fn validate_content(&self) -> bool {
        !self.text.is_empty()
    }
}

impl Into<String> for TextContent {
    fn into(self) -> String {
        self.text
    }
}

impl AddAssign<String> for TextContent {
    fn add_assign(&mut self, rhs: String) {
        self.text += &rhs;
    }
}

impl AddAssign<ContentTextDelta> for TextContent {
    fn add_assign(&mut self, rhs: ContentTextDelta) {
        self.text += &rhs.text;
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ContentTextDelta {
    text: String,
}
