use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::ops::AddAssign;

fn default_value() -> String {
    "".to_string()
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ToolUseContent {
    id: String,
    name: String,
    input: Value,
    #[serde(default = "default_value")]
    #[serde(skip_serializing)]
    _input_raw: String,
}

impl ToolUseContent {
    pub fn create_response(&self, content: impl Into<String>) -> ToolResultContent {
        ToolResultContent {
            tool_use_id: self.id.clone(),
            content: content.into(),
        }
    }

    pub fn name_is(&self, name: impl Into<String>) -> bool {
        self.name == name.into()
    }
}

impl AddAssign<ContentInputJsonDelta> for ToolUseContent {
    fn add_assign(&mut self, rhs: ContentInputJsonDelta) {
        self._input_raw += &rhs.partial_json;

        let result = serde_json::from_str::<Value>(self._input_raw.clone().as_str());
        if let Ok(value) = result {
            self.input = value;
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ToolResultContent {
    tool_use_id: String,
    content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ContentInputJsonDelta {
    partial_json: String,
}
