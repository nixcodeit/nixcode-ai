mod tools;
pub mod project;
mod utils;
mod prompts;

use crate::project::Project;
use crate::prompts::system::SYSTEM_PROMPT;
use crate::tools::fs::{CreateFileTool, DeleteFileTool, ReadTextFileTool, UpdateTextFileTool};
use crate::tools::glob::SearchGlobFilesTool;
use crate::tools::Tools;
use nixcode_llm_sdk::config::LLMConfig;
use nixcode_llm_sdk::errors::llm::LLMError;
use nixcode_llm_sdk::message::content::Content;
use nixcode_llm_sdk::message::message::Message;
use nixcode_llm_sdk::{LLMClient, MessageResponseStream, MessageResponseStreamEvent, Request, ThinkingOptions};
use std::default::Default;
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;

pub struct Nixcode {
    project: Project,
    client: LLMClient,
    model: String,
    tools: Tools,
}

impl Nixcode {
    pub fn new(project: Project, client: LLMClient) -> anyhow::Result<Self, LLMError> {
        Ok(Self {
            project,
            client,
            model: "claude-3-7-sonnet-20250219".into(),
            tools: {
                let mut tools = Tools::new();

                tools.add_tool(Arc::new(SearchGlobFilesTool {}));
                tools.add_tool(Arc::new(CreateFileTool {}));
                tools.add_tool(Arc::new(ReadTextFileTool {}));
                tools.add_tool(Arc::new(UpdateTextFileTool {}));
                tools.add_tool(Arc::new(DeleteFileTool {}));

                tools
            },
        })
    }

    pub fn new_anthropic(project: Project, config: LLMConfig) -> anyhow::Result<Self, LLMError> {
        let client = LLMClient::new_anthropic(config)?;
        Self::new(project, client)
    }

    pub fn new_openai(project: Project, config: LLMConfig) -> anyhow::Result<Self, LLMError> {
        let client = LLMClient::new_openai(config)?;
        Self::new(project, client)
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub async fn send(&self, messages: Vec<Message>) -> Result<MessageResponseStream, LLMError> {
        let mut request = Request::default()
            .with_model(self.model.clone())
            .with_max_tokens(51200)
            .with_messages(messages)
            .with_system_prompt(vec![Content::new_text(SYSTEM_PROMPT)])
            .with_thinking(ThinkingOptions::new(8192))
            .with_cache();

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
        self.tools.execute_tool(name, params, &self.project)
    }
}
