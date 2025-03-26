use serde::{Deserialize, Serialize};
use std::ops::AddAssign;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ThinkingContent {
    thinking: String,
    signature: String,
}

impl ThinkingContent {
    pub fn get_text(&self) -> String {
        self.thinking.clone()
    }
}

impl AddAssign<ContentThinkingDelta> for ThinkingContent {
    fn add_assign(&mut self, rhs: ContentThinkingDelta) {
        self.thinking += &rhs.thinking;
    }
}

impl AddAssign<ContentSignatureDelta> for ThinkingContent {
    fn add_assign(&mut self, rhs: ContentSignatureDelta) {
        self.signature += &rhs.signature;
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RedactedThinkingContent {
    data: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ContentThinkingDelta {
    thinking: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ContentSignatureDelta {
    signature: String,
}
