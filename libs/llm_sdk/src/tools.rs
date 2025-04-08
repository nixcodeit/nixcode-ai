use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "input_schema")]
    pub input: Value,
}

impl Tool {
    pub fn new(name: String, description: String, input: Value) -> Self {
        Self {
            name,
            description,
            input,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_input(mut self, input: Value) -> Self {
        self.input = input;
        self
    }
}
