use schemars::Schema;

#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input: Schema,
}

impl Tool {
    pub fn new(name: String, description: String, input: Schema) -> Self {
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

    pub fn with_input(mut self, input: Schema) -> Self {
        self.input = input;
        self
    }
}
