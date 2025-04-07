use serde::{Deserialize, Serialize};

pub mod events;
pub mod tokens;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct OpenAIPromptTokenDetails {
    pub cached_tokens: Option<usize>,
    pub audio_tokens: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct OpenAIUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    pub prompt_tokens_details: Option<OpenAIPromptTokenDetails>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct OpenAIResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub usage: Option<OpenAIUsage>,
    pub choices: Vec<OpenAIChoice>,
    pub system_fingerprint: Option<String>,
    pub service_tier: Option<String>,
}

impl OpenAIResponse {
    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }

    pub fn set_model(&mut self, model: String) {
        self.model = model;
    }

    pub fn set_object(&mut self, object: String) {
        self.object = object;
    }

    pub fn set_created(&mut self, created: i64) {
        self.created = created;
    }

    pub fn set_usage(&mut self, usage: OpenAIUsage) {
        self.usage = Some(usage);
    }

    pub fn set_choices(&mut self, choices: Vec<OpenAIChoice>) {
        self.choices = choices;
    }

    pub fn set_system_fingerprint(&mut self, system_fingerprint: String) {
        self.system_fingerprint = Some(system_fingerprint);
    }

    pub fn set_service_tier(&mut self, service_tier: String) {
        self.service_tier = Some(service_tier);
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct OpenAIChoice {
    pub index: usize,
    pub finish_reason: Option<String>,
    pub message: OpenAIMessage,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct OpenAIMessage {
    pub role: String,
    pub reasoning: String,
    pub content: String,
    pub tool_calls: Vec<OpenAIMessageToolCall>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct OpenAIMessageToolCall {
    pub id: String,
    pub r#type: String,
    pub function: OpenAIMessageFunction,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct OpenAIMessageFunction {
    pub name: String,
    pub arguments: String,
}
