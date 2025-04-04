use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::ops::AddAssign;

fn default_value() -> String {
    "".to_string()
}

#[derive(Debug, Clone, Default)]
pub enum ToolUseState {
    #[default]
    Created,
    Executing,
    Executed,
    Error,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ToolUseContent {
    pub id: String,
    pub name: String,
    pub input: Value,
    #[serde(default = "default_value")]
    #[serde(skip_serializing)]
    pub _input_raw: String,
    #[serde(skip)]
    pub _tool_execution_state: ToolUseState,
    #[serde(skip)]
    pub _tool_result: Value,
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

    pub fn get_tool_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_execute_params(&self) -> (String, Value) {
        (self.name.clone(), self.input.clone())
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn get_state(&self) -> ToolUseState {
        self._tool_execution_state.clone()
    }

    pub(crate) fn set_state(&mut self, state: ToolUseState) {
        self._tool_execution_state = state;
    }

    pub fn validate_content(&self) -> bool {
        !self.input.is_null()
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

impl ToolResultContent {
    pub fn get_tool_use_id(&self) -> String {
        self.tool_use_id.clone()
    }

    pub fn get_content(&self) -> String {
        self.content.clone()
    }

    pub fn validate_content(&self) -> bool {
        !self.content.is_empty()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ContentInputJsonDelta {
    pub(crate) partial_json: String,
}
