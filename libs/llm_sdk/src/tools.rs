use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct Tool {
    name: String,
    description: String,
    #[serde(rename = "input_schema")]
    input: Value,
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
