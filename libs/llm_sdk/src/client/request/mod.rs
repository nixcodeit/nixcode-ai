use crate::errors::llm::LLMError;
use crate::message::common::llm_message::LLMRequest;
use crate::message::content::Content;
use crate::message::message::Message;
use crate::tools::Tool;
use crate::ThinkingOptions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<ThinkingOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<Vec<Content>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

impl TryFrom<&LLMRequest> for Request {
    type Error = LLMError;

    fn try_from(request: &LLMRequest) -> Result<Self, Self::Error> {
        let mut messages = vec![];
        for message in request.messages.iter() {
            messages.push(Message::try_from(message)?);
        }

        Ok(Request {
            model: request.model.model_name().to_string(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            stream: request.stream,
            thinking: None,
            tools: request.tools.clone(),
            system: request.system.clone().map(|c| vec![Content::new_text(c)]),
            stop_sequences: request.stop_sequences.clone(),
        })
    }
}

impl Default for Request {
    fn default() -> Self {
        Request {
            model: "claude-3-7-sonnet-20250219".to_string(),
            messages: Vec::new(),
            max_tokens: None,
            temperature: None,
            stream: true,
            thinking: None,
            tools: None,
            system: None,
            stop_sequences: None,
        }
    }
}

impl Request {
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    pub fn with_thinking(mut self, thinking: ThinkingOptions) -> Self {
        self.thinking = Some(thinking);
        self
    }

    pub fn with_messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }

    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn with_system_prompt(mut self, system: Vec<Content>) -> Self {
        self.system = Some(system);
        self
    }

    pub fn get_messages(&self) -> &Vec<Message> {
        &self.messages
    }

    pub fn get_system(&self) -> &Option<Vec<Content>> {
        &self.system
    }

    pub fn get_tools(&self) -> &Option<Vec<Tool>> {
        &self.tools
    }

    pub fn get_model(&self) -> &str {
        &self.model
    }

    pub fn get_max_tokens(&self) -> Option<u32> {
        self.max_tokens
    }

    pub fn get_temperature(&self) -> Option<f32> {
        self.temperature
    }
}
