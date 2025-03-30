pub mod config;
pub mod project;
mod prompts;
mod tools;
mod utils;

use crate::config::Config;
use crate::project::Project;
use crate::prompts::system::SYSTEM_PROMPT;
use crate::tools::fs::create_file::CreateFileTool;
use crate::tools::fs::delete_file::DeleteFileTool;
use crate::tools::fs::delete_text_file_partial::DeleteTextFilePartialTool;
use crate::tools::fs::read_text_file::ReadTextFileTool;
use crate::tools::fs::update_text_file_partial::UpdateTextFilePartialTool;
use crate::tools::fs::write_text_file::WriteTextFileTool;
use crate::tools::git::git_add::GitAddTool;
use crate::tools::git::git_diff::GitDiffTool;
use crate::tools::git::git_log::GitLogTool;
use crate::tools::git::git_stash_apply::GitStashApplyTool;
use crate::tools::git::git_stash_drop::GitStashDropTool;
use crate::tools::git::git_stash_list::GitStashListTool;
use crate::tools::git::git_stash_save::GitStashSaveTool;
use crate::tools::git::git_status::GitStatusTool;
use crate::tools::glob::search_glob_files::SearchGlobFilesTool;
use crate::tools::prompt::get_project_analysis_prompt::GetProjectAnalysisPromptTool;
use crate::tools::search::replace_content::ReplaceContentTool;
use crate::tools::search::search_content::SearchContentTool;
use crate::tools::Tools;
use anyhow::Result;
use nixcode_llm_sdk::config::LLMConfig;
use nixcode_llm_sdk::errors::llm::LLMError;
use nixcode_llm_sdk::message::content::Content;
use nixcode_llm_sdk::message::message::Message;
use nixcode_llm_sdk::{LLMClient, MessageResponseStream, MessageResponseStreamEvent, Request};
use secrecy::SecretString;
use std::default::Default;
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;

pub struct Nixcode {
    project: Arc<Project>,
    client: LLMClient,
    model: String,
    tools: Tools,
    config: Config,
}

impl Nixcode {
    pub fn new(
        project: Project,
        client: LLMClient,
        config: Config,
    ) -> anyhow::Result<Self, LLMError> {
        let has_init_analysis = project.has_init_analysis();
        let model = config.get_model_for_provider(&config.llm.default_provider);
        let has_repo_path = project.has_repo_path();

        Ok(Self {
            project: Arc::new(project),
            client,
            model,
            config,
            tools: {
                let mut tools = Tools::new();

                tools.add_tool(Arc::new(SearchGlobFilesTool {}));
                tools.add_tool(Arc::new(CreateFileTool {}));
                tools.add_tool(Arc::new(ReadTextFileTool {}));
                tools.add_tool(Arc::new(WriteTextFileTool {}));
                tools.add_tool(Arc::new(UpdateTextFilePartialTool {}));
                tools.add_tool(Arc::new(DeleteFileTool {}));
                tools.add_tool(Arc::new(DeleteTextFilePartialTool {}));
                tools.add_tool(Arc::new(SearchContentTool {}));
                tools.add_tool(Arc::new(ReplaceContentTool {}));

                if has_repo_path {
                    tools.add_tool(Arc::new(GitAddTool {}));
                    // tools.add_tool(Arc::new(GitCommitTool {}));
                    tools.add_tool(Arc::new(GitStatusTool {}));
                    tools.add_tool(Arc::new(GitDiffTool {}));
                    tools.add_tool(Arc::new(GitStashSaveTool {}));
                    tools.add_tool(Arc::new(GitStashApplyTool {}));
                    tools.add_tool(Arc::new(GitStashListTool {}));
                    tools.add_tool(Arc::new(GitStashDropTool {}));
                    tools.add_tool(Arc::new(GitLogTool {}));
                }

                if !has_init_analysis {
                    tools.add_tool(Arc::new(GetProjectAnalysisPromptTool {}));
                }

                tools
            },
        })
    }

    /// Creates a new Nixcode instance with configuration from files or environment
    pub fn new_from_env(project: Project) -> anyhow::Result<Self, LLMError> {
        // Try to load configuration, fallback to defaults if it fails
        let config = Config::load().unwrap_or_else(|_| Config::new());
        Self::new_with_config(project, config)
    }

    /// Creates a new Nixcode instance with provided configuration
    pub fn new_with_config(project: Project, config: Config) -> anyhow::Result<Self, LLMError> {
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
    pub fn new_anthropic(project: Project, config: LLMConfig) -> anyhow::Result<Self, LLMError> {
        let app_config = Config::load().unwrap_or_else(|_| Config::new());
        let client = LLMClient::new_anthropic(config)?;
        Self::new(project, client, app_config)
    }

    pub fn new_openai(project: Project, config: LLMConfig) -> anyhow::Result<Self, LLMError> {
        let app_config = Config::load().unwrap_or_else(|_| Config::new());
        let client = LLMClient::new_openai(config)?;
        Self::new(project, client, app_config)
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub async fn send(&self, messages: Vec<Message>) -> Result<MessageResponseStream, LLMError> {
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

    pub async fn execute_tool(
        &self,
        name: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.tools
            .execute_tool(name, params, self.project.clone())
            .await
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
}
