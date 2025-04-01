pub mod config;
pub mod events;
pub mod project;
mod prompts;
mod tools;
mod utils;

use crate::config::Config;
use crate::events::NixcodeEvent;
use crate::project::Project;
use crate::prompts::system::SYSTEM_PROMPT;
use crate::tools::fs::create_file::CreateFileTool;
use crate::tools::fs::delete_file::DeleteFileTool;
use crate::tools::fs::read_text_file::ReadTextFileTool;
use crate::tools::fs::write_text_file::WriteTextFileTool;
use crate::tools::git::git_add::GitAddTool;
use crate::tools::git::git_branch_create::GitBranchCreateTool;
use crate::tools::git::git_branch_delete::GitBranchDeleteTool;
use crate::tools::git::git_branches::GitBranchesTool;
use crate::tools::git::git_commit::GitCommitTool;
use crate::tools::git::git_diff::GitDiffTool;
use crate::tools::git::git_log::GitLogTool;
use crate::tools::git::git_stash_apply::GitStashApplyTool;
use crate::tools::git::git_stash_drop::GitStashDropTool;
use crate::tools::git::git_stash_list::GitStashListTool;
use crate::tools::git::git_stash_save::GitStashSaveTool;
use crate::tools::git::git_status::GitStatusTool;
use crate::tools::git::git_tag_create::GitTagCreateTool;
use crate::tools::git::git_tags_list::GitTagsListTool;
use crate::tools::glob::search_glob_files::SearchGlobFilesTool;
use crate::tools::prompt::get_project_analysis_prompt::GetProjectAnalysisPromptTool;
use crate::tools::search::replace_content::ReplaceContentTool;
use crate::tools::search::search_content::SearchContentTool;
use crate::tools::Tools;
use anyhow::Result;
use nixcode_llm_sdk::config::LLMConfig;
use nixcode_llm_sdk::errors::llm::LLMError;
use nixcode_llm_sdk::message::content::tools::{ToolResultContent, ToolUseContent, ToolUseState};
use nixcode_llm_sdk::message::content::Content;
use nixcode_llm_sdk::message::message::Message;
use nixcode_llm_sdk::message::message::Message::Assistant;
use nixcode_llm_sdk::message::response::MessageResponse;
use nixcode_llm_sdk::message::usage::Usage;
use nixcode_llm_sdk::{ErrorContent, LLMClient, MessageResponseStreamEvent, Request};
use secrecy::SecretString;
use std::default::Default;
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;

pub struct Nixcode {
    project: Arc<Project>,
    client: LLMClient,
    model: String,
    tools: Tools,
    config: Config,
    messages: RwLock<Vec<Message>>,
    usage: RwLock<Usage>,
    tools_to_execute: RwLock<Vec<ToolUseContent>>,
    tools_results: RwLock<Vec<ToolResultContent>>,
    last_message_response: RwLock<Option<MessageResponse>>,
    llm_error: RwLock<Option<ErrorContent>>,
    is_waiting: RwLock<bool>,
    tx: UnboundedSender<NixcodeEvent>,
}

pub type NewNixcodeResult = (UnboundedReceiver<NixcodeEvent>, Nixcode);

impl Nixcode {
    pub fn new(
        project: Project,
        client: LLMClient,
        config: Config,
    ) -> Result<NewNixcodeResult, LLMError> {
        let has_init_analysis = project.has_init_analysis();
        let model = config.get_model_for_provider(&config.llm.default_provider);
        let has_repo_path = project.has_repo_path();

        let (tx, rx) = unbounded_channel::<NixcodeEvent>();
        let nixcode = Self {
            project: Arc::new(project),
            client,
            model,
            config: config.clone(),
            messages: RwLock::new(vec![]),
            usage: RwLock::new(Usage::default()),
            llm_error: RwLock::new(None),
            last_message_response: RwLock::new(None),
            tools_results: RwLock::new(vec![]),
            tools_to_execute: RwLock::new(vec![]),
            is_waiting: RwLock::new(false),
            tx,
            tools: {
                let mut tools = Tools::new();

                // Register all tools unconditionally
                tools.add_tool(Arc::new(SearchGlobFilesTool {}));
                tools.add_tool(Arc::new(CreateFileTool {}));
                tools.add_tool(Arc::new(ReadTextFileTool {}));
                tools.add_tool(Arc::new(WriteTextFileTool {}));
                // tools.add_tool(Arc::new(UpdateTextFilePartialTool {}));
                tools.add_tool(Arc::new(DeleteFileTool {}));
                // tools.add_tool(Arc::new(DeleteTextFilePartialTool {}));
                tools.add_tool(Arc::new(SearchContentTool {}));
                tools.add_tool(Arc::new(ReplaceContentTool {}));

                if has_repo_path {
                    tools.add_tool(Arc::new(GitAddTool {}));
                    tools.add_tool(Arc::new(GitCommitTool {}));
                    tools.add_tool(Arc::new(GitStatusTool {}));
                    tools.add_tool(Arc::new(GitDiffTool {}));
                    tools.add_tool(Arc::new(GitStashSaveTool {}));
                    tools.add_tool(Arc::new(GitStashApplyTool {}));
                    tools.add_tool(Arc::new(GitStashListTool {}));
                    tools.add_tool(Arc::new(GitStashDropTool {}));
                    tools.add_tool(Arc::new(GitLogTool {}));
                    tools.add_tool(Arc::new(GitBranchesTool {}));
                    tools.add_tool(Arc::new(GitBranchCreateTool {}));
                    tools.add_tool(Arc::new(GitBranchDeleteTool {}));
                    tools.add_tool(Arc::new(GitTagCreateTool {}));
                    tools.add_tool(Arc::new(GitTagsListTool {}));
                }

                if !has_init_analysis {
                    tools.add_tool(Arc::new(GetProjectAnalysisPromptTool {}));
                }

                tools
            },
        };

        Ok((rx, nixcode))
    }

    /// Creates a new Nixcode instance with configuration from files or environment
    pub fn new_from_env(project: Project) -> anyhow::Result<NewNixcodeResult, LLMError> {
        // Try to load configuration, fallback to defaults if it fails
        let config = Config::load().unwrap_or_else(|_| Config::new());
        Self::new_with_config(project, config)
    }

    /// Creates a new Nixcode instance with provided configuration
    pub fn new_with_config(
        project: Project,
        config: Config,
    ) -> anyhow::Result<NewNixcodeResult, LLMError> {
        let provider = &config.llm.default_provider;

        // Try to get API key for the provider
        let api_key_result = config.get_api_key_for_provider(provider);

        match (provider.as_str(), api_key_result) {
            // Anthropic with available API key
            ("anthropic", Ok(api_key)) => {
                let llm_config = LLMConfig { api_key };
                let client = LLMClient::new_anthropic(llm_config)?;
                Self::new(project, client, config)
            }
            // OpenAI with available API key
            ("openai", Ok(api_key)) => {
                let llm_config = LLMConfig { api_key };
                let client = LLMClient::new_openai(llm_config)?;
                Self::new(project, client, config)
            }
            // Fallback to environment variables for Anthropic
            (_, _) => {
                let api_key = env::var("ANTHROPIC_API_KEY").map_err(|_| LLMError::MissingAPIKey)?;

                let llm_config = LLMConfig {
                    api_key: SecretString::new(api_key.into()),
                };

                let client = LLMClient::new_anthropic(llm_config)?;
                Self::new(project, client, config)
            }
        }
    }

    // Legacy methods kept for compatibility
    pub fn new_anthropic(
        project: Project,
        config: LLMConfig,
    ) -> anyhow::Result<NewNixcodeResult, LLMError> {
        let app_config = Config::load().unwrap_or_else(|_| Config::new());
        let client = LLMClient::new_anthropic(config)?;
        Self::new(project, client, app_config)
    }

    pub fn new_openai(
        project: Project,
        config: LLMConfig,
    ) -> anyhow::Result<NewNixcodeResult, LLMError> {
        let app_config = Config::load().unwrap_or_else(|_| Config::new());
        let client = LLMClient::new_openai(config)?;
        Self::new(project, client, app_config)
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub async fn is_waiting(&self) -> bool {
        *self.is_waiting.read().await
    }

    pub async fn send(self: Arc<Self>, messages: Vec<Message>) {
        let mut system_prompt = vec![Content::new_text(SYSTEM_PROMPT)];
        let project_init_analysis_content = self.project.get_project_init_analysis_content();
        if let Some(content) = project_init_analysis_content {
            let content = format!("{}\n\n{}", "File: .nixcode/init.md", content);
            system_prompt.push(Content::new_text(content));
        }

        let mut request = Request::default()
            .with_model(self.model.clone())
            .with_max_tokens(51200)
            .with_messages(messages)
            .with_system_prompt(system_prompt)
            // .with_thinking(ThinkingOptions::new(8192))
            .with_cache();

        // Use enabled_tools instead of all tools
        let enabled_tools = self.tools.get_enabled_tools(&self.config);
        if !enabled_tools.is_empty() {
            request = request.with_tools(enabled_tools);
        }
        let nixcode_event_sender = self.tx.clone();

        *self.is_waiting.write().await = true;

        nixcode_event_sender
            .send(NixcodeEvent::GeneratingResponse)
            .ok();
        let response = self.client.send(request).await;

        if let Err(err) = response {
            *self.is_waiting.write().await = false;
            *self.llm_error.write().await = Some(err.clone().into());
            nixcode_event_sender.send(NixcodeEvent::Error(err)).ok();
            return;
        }

        let mut stream = response.unwrap();

        *self.last_message_response.write().await = Some(MessageResponse::default());
        self.add_message(Assistant(vec![])).await;

        tokio::spawn({
            let x = self.clone();

            async move {
                while let Some(event) = stream.recv().await {
                    x.handle_response_event(event.clone()).await;
                }

                *self.is_waiting.write().await = false;
                nixcode_event_sender
                    .send(NixcodeEvent::GeneratedResponse)
                    .ok();

                x.execute_tools().await;
            }
        });
    }

    pub async fn get_tools_to_execute(self: &Arc<Self>) -> Vec<ToolUseContent> {
        self.tools_to_execute.read().await.clone()
    }

    pub async fn set_waiting(&self, new_val: bool) {
        *self.is_waiting.write().await = new_val;
    }

    async fn add_message(&self, message: Message) {
        self.messages.write().await.push(message);
        self.tx.send(NixcodeEvent::NewMessage).ok();
    }

    pub async fn send_message(self: Arc<Self>, message: Option<Message>) {
        if let Some(message) = message {
            self.add_message(message).await;
        }

        let messages = self.messages.read().await.clone();

        self.send(messages).await
    }

    pub async fn execute_tool(self: Arc<Self>, tool: ToolUseContent) {
        let (name, props) = tool.get_execute_params();

        if !self.config.is_tool_enabled(name.as_str()) {
            return;
        }

        self.clone().start_tool(tool.clone()).await;

        let result = self
            .tools
            .execute_tool(name.as_str(), props, self.project.clone())
            .await;

        let result = if let Ok(value) = result {
            let value = serde_json::from_value(value).unwrap_or_else(|e| e.to_string());
            tool.create_response(value)
        } else {
            tool.create_response("Error executing tool".to_string())
        };

        self.clone().tool_finished(result).await;
    }

    pub fn has_init_analysis(&self) -> bool {
        self.project.has_init_analysis()
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn get_model(&self) -> &str {
        &self.model
    }

    pub fn get_project(&self) -> Arc<Project> {
        self.project.clone()
    }

    pub async fn get_messages(&self) -> Vec<Message> {
        self.messages.read().await.clone()
    }

    pub async fn get_error(&self) -> Option<ErrorContent> {
        self.llm_error.read().await.clone()
    }

    pub async fn get_usage(&self) -> Usage {
        self.usage.read().await.clone()
    }

    pub async fn send_tools_results(self: Arc<Self>) {
        let contents = self.tools_results.read().await.clone();
        self.tools_results.write().await.clear();
        self.tools_to_execute.write().await.clear();

        let message = Message::User(Content::new_tool_results(contents));

        self.send_message(Some(message)).await;
    }

    pub async fn handle_response_event(self: &Arc<Self>, message: MessageResponseStreamEvent) {
        let mut last_message_response = self.last_message_response.write().await;
        let mut messages = self.messages.write().await;

        if last_message_response.is_none() || messages.last().is_none() {
            return;
        }

        let mut usage = self.usage.write().await;
        let last_message = messages.last_mut().unwrap();
        let last_response = last_message_response.as_mut().unwrap();
        let mut message_updated = false;
        match message {
            MessageResponseStreamEvent::MessageStart(msg) => {
                *last_response += msg;
                *usage += last_response.usage.clone();
                message_updated = true;
            }
            MessageResponseStreamEvent::MessageDelta(delta) => {
                usage.output_tokens += delta.get_usage().output_tokens;
                *last_response += delta;
                message_updated = true;
            }
            MessageResponseStreamEvent::ContentBlockStart(content) => {
                *last_response += content;
                message_updated = true;
            }
            MessageResponseStreamEvent::ContentBlockDelta(delta) => {
                let index = delta.get_index();
                *last_response += delta;

                match last_response.get_content(index) {
                    Content::ToolUse(_) => (),
                    _ => { message_updated = true; },
                }
            }
            MessageResponseStreamEvent::ContentBlockStop(content) => {
                if let Content::ToolUse(tool_use) = last_response.get_content(content.index) {
                    self.clone().add_tool_to_execute(tool_use).await;
                }
                message_updated = true;
            }
            MessageResponseStreamEvent::Error { error } => {
                *self.llm_error.write().await = Some(error.clone());
            }
            _ => (),
        }

        if message_updated {
            self.tx.send(NixcodeEvent::MessageUpdated).ok();
        }

        last_message.set_content(last_response.content.clone());
    }

    async fn execute_tools(self: &Arc<Self>) {
        let tools = self.get_tools_to_execute().await;

        if tools.is_empty() {
            return;
        }

        for tool in tools {
            tokio::spawn({
                let nixcode = self.clone();
                async move {
                    nixcode.execute_tool(tool).await;
                }
            });
        }
    }

    pub async fn remove_last_message(self: &Arc<Self>) {
        if self.is_waiting().await {
            return;
        }

        let mut messages = self.messages.write().await;
        if let Some(Assistant(content)) = messages.last() {
            if content.is_empty() {
                messages.pop();
            }
        }

        messages.pop();
        *self.llm_error.write().await = None;
    }

    pub async fn reset(self: &Arc<Self>) -> Result<()> {
        if self.is_waiting().await {
            return Err(anyhow::anyhow!("Cannot reset while waiting for response"));
        }

        *self.last_message_response.write().await = None;
        self.tools_results.write().await.clear();
        self.tools_to_execute.write().await.clear();
        self.messages.write().await.clear();
        *self.usage.write().await = Usage::default();

        Ok(())
    }

    pub async fn retry_last_message(self: &Arc<Self>) {
        if self.is_waiting().await {
            return;
        }

        let mut messages = self.messages.write().await;
        loop {
            let last_message = messages.last();
            if last_message.is_none() {
                break;
            }

            if let Assistant(_) = last_message.unwrap() {
                messages.pop();
                continue;
            }

            break;
        }

        if messages.len() == 0 {
            return;
        }

        drop(messages);
        self.clone().send_message(None).await
    }

    pub async fn add_tool_to_execute(self: &Arc<Self>, content: ToolUseContent) {
        self.tools_to_execute.write().await.push(content.clone());
    }

    pub async fn start_tool(self: &Arc<Self>, tool: ToolUseContent) {
        let mut messages = self.messages.write().await;
        let last_message = messages.last_mut().unwrap();
        last_message.set_tool_state(tool.get_id(), ToolUseState::Executing);
    }

    pub async fn tool_finished(self: &Arc<Self>, result: ToolResultContent) {
        let tool_id = result.get_tool_use_id();
        self.tools_results.write().await.push(result.clone());

        let tools_results = self.tools_results.read().await.clone();
        let tools_to_execute = self.tools_to_execute.read().await.clone();
        let mut messages = self.messages.write().await;
        let last_message = messages.last_mut().unwrap();
        last_message.set_tool_state(tool_id, ToolUseState::Executed);

        self.tx.send(NixcodeEvent::ToolEnd(result)).ok();

        if tools_results.len() != tools_to_execute.len() {
            return;
        }

        self.tx.send(NixcodeEvent::ToolsFinished).ok();
    }
}
