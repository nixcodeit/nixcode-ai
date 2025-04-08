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
use crate::tools::commands::cargo_build::CargoBuildTool;
use crate::tools::commands::cargo_fix::CargoFixTool;
use crate::tools::commands::cargo_fmt::CargoFmtTool;
use crate::tools::commands::cargo_test::CargoTestTool;
use crate::tools::fs::create_file::CreateFileTool;
use crate::tools::fs::delete_file::DeleteFileTool;
use crate::tools::fs::read_text_file::ReadTextFileTool;
use crate::tools::fs::write_text_file::WriteTextFileTool;
use crate::tools::git::git_add::GitAddTool;
use crate::tools::git::git_branch_create::GitBranchCreateTool;
use crate::tools::git::git_branch_delete::GitBranchDeleteTool;
use crate::tools::git::git_branches::GitBranchesTool;
use crate::tools::git::git_checkout::GitCheckoutTool;
use crate::tools::git::git_commit::GitCommitTool;
use crate::tools::git::git_diff::GitDiffTool;
use crate::tools::git::git_log::GitLogTool;
use crate::tools::git::git_pull::GitPullTool;
use crate::tools::git::git_push::GitPushTool;
use crate::tools::git::git_reset::GitResetTool;
use crate::tools::git::git_stash_apply::GitStashApplyTool;
use crate::tools::git::git_stash_drop::GitStashDropTool;
use crate::tools::git::git_stash_list::GitStashListTool;
use crate::tools::git::git_stash_save::GitStashSaveTool;
use crate::tools::git::git_status::GitStatusTool;
use crate::tools::git::git_tag_create::GitTagCreateTool;
use crate::tools::git::git_tags_list::GitTagsListTool;
use crate::tools::github::issue_details::GithubIssueDetailsTool;
use crate::tools::github::list_issues::GithubIssuesListTool;
use crate::tools::glob::search_glob_files::SearchGlobFilesTool;
use crate::tools::prompt::get_project_analysis_prompt::GetProjectAnalysisPromptTool;
use crate::tools::search::replace_content::ReplaceContentTool;
use crate::tools::search::search_content::SearchContentTool;
use crate::tools::Tools;
use anyhow::Result;
use futures::future::join_all;
use nixcode_llm_sdk::client::LLMClient;
use nixcode_llm_sdk::config::HttpClientOptions;
use nixcode_llm_sdk::errors::llm::LLMError;
use nixcode_llm_sdk::message::anthropic::events::ErrorEventContent;
use nixcode_llm_sdk::message::common::llm_message::{LLMEvent, LLMMessage, LLMRequest};
use nixcode_llm_sdk::message::content::Content;
use nixcode_llm_sdk::message::usage::AnthropicUsage;
use nixcode_llm_sdk::models::llm_model::LLMModel;
use secrecy::SecretString;
use std::default::Default;
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;

pub struct Nixcode {
    project: Arc<Project>,
    client: LLMClient,
    model: &'static LLMModel,
    tools: Tools,
    config: Config,
    messages: RwLock<Vec<LLMMessage>>,
    usage: RwLock<AnthropicUsage>,
    llm_error: RwLock<Option<ErrorEventContent>>,
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
            usage: RwLock::new(AnthropicUsage::default()),
            llm_error: RwLock::new(None),
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

                // Add cargo tools
                tools.add_tool(Arc::new(CargoBuildTool {}));
                tools.add_tool(Arc::new(CargoFmtTool {}));
                tools.add_tool(Arc::new(CargoFixTool {}));
                tools.add_tool(Arc::new(CargoTestTool {}));

                // GitHub tools
                tools.add_tool(Arc::new(GithubIssuesListTool {}));
                tools.add_tool(Arc::new(GithubIssueDetailsTool {}));

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
                    // Add new git tools
                    tools.add_tool(Arc::new(GitCheckoutTool {}));
                    tools.add_tool(Arc::new(GitPullTool {}));
                    tools.add_tool(Arc::new(GitPushTool {}));
                    tools.add_tool(Arc::new(GitResetTool {}));
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
    pub fn new_from_env(project: Project) -> Result<NewNixcodeResult, LLMError> {
        // Try to load configuration, fallback to defaults if it fails
        let config = Config::load().unwrap_or_else(|_| Config::new());
        Self::new_with_config(project, config)
    }

    /// Creates a new Nixcode instance with provided configuration
    pub fn new_with_config(
        mut project: Project,
        config: Config,
    ) -> Result<NewNixcodeResult, LLMError> {
        if let Ok(token) = config.get_github_token() {
            let client = octocrab::OctocrabBuilder::new()
                .user_access_token(token.clone())
                .build();

            if let Err(e) = client {
                log::error!("Failed to initialize Octocrab client: {}", e);

                return Err(LLMError::Generic(format!(
                    "Failed to initialize Octocrab client: {}",
                    e
                )));
            }

            octocrab::initialise(client.unwrap());
        }

        project.set_github(&config.github);

        let provider = &config.llm.default_provider;

        // Try to get API key for the provider
        let api_key_result = config.get_api_key_for_provider(provider);

        match (provider.as_str(), api_key_result) {
            // Anthropic with available API key
            ("anthropic", Ok(api_key)) => {
                let llm_config = HttpClientOptions::new_anthropic(api_key);
                let client = LLMClient::new_anthropic(llm_config)?;
                Self::new(project, client, config)
            }
            // OpenAI with available API key
            ("openai", Ok(api_key)) => {
                let llm_config = HttpClientOptions::new_openai(api_key);
                let client = LLMClient::new_openai(llm_config)?;
                Self::new(project, client, config)
            }
            ("groq", Ok(api_key)) => {
                let llm_config = HttpClientOptions::new_groq(api_key);
                let client = LLMClient::new_openai(llm_config)?;
                Self::new(project, client, config)
            }
            ("open_router", Ok(api_key)) => {
                let llm_config = HttpClientOptions::new_openrouter(api_key);
                let client = LLMClient::new_openai(llm_config)?;
                Self::new(project, client, config)
            }
            // Fallback to environment variables for Anthropic
            (_, _) => {
                let api_key = env::var("ANTHROPIC_API_KEY").map_err(|_| LLMError::MissingAPIKey)?;

                let llm_config = HttpClientOptions::new_anthropic(SecretString::from(api_key));

                let client = LLMClient::new_anthropic(llm_config)?;
                Self::new(project, client, config)
            }
        }
    }

    pub fn with_model(mut self, model: &'static LLMModel) -> Self {
        self.model = model;
        self
    }

    pub async fn is_waiting(&self) -> bool {
        *self.is_waiting.read().await
    }

    pub async fn send(self: Arc<Self>, messages: Vec<LLMMessage>) {
        let mut system_prompt = vec![Content::new_text(SYSTEM_PROMPT)];
        let project_init_analysis_content = self.project.get_project_init_analysis_content();
        let tools = self.tools.get_enabled_tools(&self.config);

        if let Some(content) = project_init_analysis_content {
            let content = format!("<file path=\".nixcode/init.md\">{}</file>", content);
            system_prompt.push(Content::new_text(content));
        }

        let default_temperature = 0.2;

        let system = system_prompt
            .iter()
            .flat_map(|c| c.get_text())
            .map(|c| c.text)
            .collect::<Vec<String>>()
            .join("\n\n");

        let system = if system.is_empty() {
            None
        } else {
            Some(system)
        };

        // Create request with parameters based on model capabilities
        let request = LLMRequest {
            model: self.model,
            messages,
            system,
            max_tokens: Some(8192),
            temperature: Some(default_temperature),
            tools: Some(tools),
            stream: true,
            provider_params: None,
            // stop_sequences: Some(vec![
            //     "[/function_call]".into(),
            //     "</function_call>".into(),
            //     "</|function|>".into(),
            // ]),
            stop_sequences: None,
        };

        log::debug!("LLMRequest {:?}", request);

        let nixcode_event_sender = self.tx.clone();

        *self.is_waiting.write().await = true;
        *self.llm_error.write().await = None;

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

        self.add_message(LLMMessage::new("assistant")).await;

        tokio::spawn({
            let nixcode = self.clone();

            async move {
                while let Some(event) = stream.recv().await {
                    nixcode.handle_response_event(event).await;
                }

                *self.is_waiting.write().await = false;
                nixcode_event_sender
                    .send(NixcodeEvent::GeneratedResponse)
                    .ok();
            }
        });
    }

    pub async fn set_waiting(&self, new_val: bool) {
        *self.is_waiting.write().await = new_val;
    }

    async fn add_message(&self, message: LLMMessage) {
        self.messages.write().await.push(message);
        self.tx.send(NixcodeEvent::NewMessage).ok();
    }

    pub async fn send_message(self: Arc<Self>, message: Option<LLMMessage>) {
        if let Some(message) = message {
            self.add_message(message).await;
        }

        let messages = self.messages.read().await.clone();

        self.send(messages).await
    }

    pub fn has_init_analysis(&self) -> bool {
        self.project.has_init_analysis()
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn get_model(&self) -> &'static LLMModel {
        &self.model
    }

    pub fn get_project(&self) -> Arc<Project> {
        self.project.clone()
    }

    pub async fn get_messages(&self) -> Vec<LLMMessage> {
        self.messages.read().await.clone()
    }

    pub async fn get_error(&self) -> Option<ErrorEventContent> {
        self.llm_error.read().await.clone()
    }

    pub async fn get_usage(&self) -> AnthropicUsage {
        self.usage.read().await.clone()
    }

    pub async fn send_tools_results(self: Arc<Self>) {
        self.send_message(None).await;
    }

    pub async fn handle_response_event(self: &Arc<Self>, message: LLMEvent) {
        let mut messages = self.messages.write().await;
        let last_message_response = messages.last_mut();

        if last_message_response.is_none() || messages.last().is_none() {
            log::error!("No last response to modify");
            return;
        }

        let last_message = messages.last_mut().unwrap();
        log::debug!("LLMMESSAGE EVENT {:?}", message);

        match message {
            LLMEvent::MessageStart => {}
            LLMEvent::MessageUpdate(msg) => {
                *last_message = msg;
                self.tx.send(NixcodeEvent::MessageUpdated).ok();
            }
            LLMEvent::Error(err) => {
                *self.llm_error.write().await = Some(err.clone().into());
                self.tx.send(NixcodeEvent::Error(err)).ok();
            }
            LLMEvent::MessageComplete => {}
        }
    }

    pub async fn execute_tools(self: &Arc<Self>) {
        log::debug!("nixcode::execute_tools");
        let messages = self.messages.read().await;
        log::debug!("get last message");
        let message = messages.last();
        if message.is_none() {
            log::debug!("message is none");
            return;
        }
        let message = message.unwrap();

        if message.tool_calls.is_none() {
            log::debug!("tool_calls is none");
            return;
        }

        let tools = message.tool_calls.clone().unwrap();
        drop(messages);

        if tools.is_empty() {
            log::debug!("tool_calls is empty");
            return;
        }

        let mut join_handles = vec![];
        log::debug!("add new user message");
        self.messages.write().await.push(LLMMessage::user());
        log::debug!("got last message");

        for tool in tools {
            let handle = tokio::spawn({
                let nixcode = self.clone();
                async move {
                    log::debug!("Starting task for tool: {:?}", tool);
                    let (name, props) = tool.get_execute_params();

                    if !nixcode.config.is_tool_enabled(name.as_str()) {
                        log::warn!("Tool {} is not enabled", name);
                        return;
                    }

                    log::debug!("Executing tool: {:?}", tool);
                    let result = nixcode
                        .tools
                        .execute_tool(name.as_str(), props, nixcode.project.clone())
                        .await;

                    let res = if let Ok(value) = result {
                        let value = serde_json::from_value(value).unwrap_or_else(|e| e.to_string());
                        tool.create_response(value)
                    } else {
                        tool.create_response("Error executing tool".to_string())
                    };

                    log::debug!("Tool result: {:?}", res);

                    let mut messages = nixcode.messages.write().await;
                    let message = messages.last_mut().unwrap();
                    message.add_tool_result(res);
                    log::debug!("Tool finished");
                    drop(messages);
                }
            });

            join_handles.push(handle);
        }

        log::debug!("Waiting for all tool tasks to finish");
        join_all(join_handles).await;
        log::debug!("All tool tasks finished");
        self.tx.send(NixcodeEvent::ToolsFinished).ok();
    }

    pub async fn remove_last_message(self: &Arc<Self>) {
        if self.is_waiting().await {
            return;
        }

        let mut messages = self.messages.write().await;
        if let Some(message) = messages.last() {
            if message.is_empty() {
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

        self.messages.write().await.clear();
        *self.usage.write().await = AnthropicUsage::default();
        *self.llm_error.write().await = None;

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

            if last_message.unwrap().role == "assistant" {
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
}