mod tools;
mod project;

use crate::tools::glob::SearchGlobFilesTool;
use crate::tools::Tools;
use eventsource_stream::Eventsource;
use nixcode_llm_sdk::config::LLMConfig;
use nixcode_llm_sdk::errors::llm::LLMError;
use nixcode_llm_sdk::message::message::Message;
use nixcode_llm_sdk::{LLMClient, LLMClientImpl, MessageResponseStream, MessageResponseStreamEvent, Request};
use std::default::Default;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;

pub struct Nixcode {
    working_directory: PathBuf,
    client: LLMClient,
    model: String,
    tools: Tools,
}

impl Nixcode {
    pub fn new(client: LLMClient) -> anyhow::Result<Self, LLMError> {
        Ok(Self {
            working_directory: Default::default(),
            client,
            model: "claude-3-7-sonnet-20250219".into(),
            tools: {
                let mut tools = Tools::new();

                tools.add_tool(Arc::new(SearchGlobFilesTool {}));

                tools
            },
        })
    }

    pub fn new_anthropic(config: LLMConfig) -> anyhow::Result<Self, LLMError> {
        let client = LLMClient::new_anthropic(config)?;
        Self::new(client)
    }

    pub fn new_openai(config: LLMConfig) -> anyhow::Result<Self, LLMError> {
        let client = LLMClient::new_openai(config)?;
        Self::new(client)
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_working_directory(mut self, working_directory: impl AsRef<Path>) -> Self {
        self.working_directory = working_directory.as_ref().to_path_buf();
        self
    }

    pub async fn send(&self, messages: Vec<Message>) -> Result<MessageResponseStream, LLMError> {
        let mut request = Request::default()
            .with_model(self.model.clone())
            .with_max_tokens(8192)
            .with_messages(messages);

        if !self.tools.is_empty() {
            request = request.with_tools(self.tools.get_all_tools());
        }

        let mut stream = self.client.send(request).await?;

        let (tx, rx) = unbounded_channel::<MessageResponseStreamEvent>();

        tokio::spawn(async move {
            while let Some(event) = stream.recv().await {
                tx.send(event).ok();
            }
        });

        Ok(rx)
    }

    pub fn execute_tool(
        &self,
        name: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        self.tools.execute_tool(name, params)
    }
}
