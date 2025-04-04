use anyhow::Result;
use directories::ProjectDirs;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use toml;

/// Default model for Anthropic
const DEFAULT_ANTHROPIC_MODEL: &str = "claude-3-7-sonnet-20250219";
/// Default model for OpenAI
const DEFAULT_OPENAI_MODEL: &str = "gpt-4o-mini";
const DEFAULT_GROQ_MODEL: &str = "gwen-qwq-32b";

/// The Config struct represents the application configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// LLM provider settings
    #[serde(default)]
    pub llm: LLMSettings,

    /// Provider-specific settings
    #[serde(default)]
    pub providers: Providers,

    /// Tool configuration
    #[serde(default)]
    pub tools: ToolsConfig,
}

/// LLM general settings
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LLMSettings {
    /// Default LLM provider to use
    #[serde(default = "default_provider")]
    pub default_provider: String,

    /// Default model to use (provider-specific defaults will be used if not specified)
    pub default_model: Option<String>,
}

fn default_provider() -> String {
    "anthropic".to_string()
}

/// Provider-specific settings
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Providers {
    /// Anthropic-specific settings
    #[serde(default)]
    pub anthropic: ProviderSettings,

    /// OpenAI-specific settings
    #[serde(default)]
    pub openai: ProviderSettings,

    /// Groq-specific settings
    #[serde(default)]
    pub groq: ProviderSettings,
}

/// Settings for a specific provider
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProviderSettings {
    /// API key for the provider (can use ${ENV_VAR} syntax)
    pub api_key: Option<String>,

    /// Default model for this provider
    pub default_model: Option<String>,
}

/// Tool configuration
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ToolsConfig {
    /// Enable all tools by default
    #[serde(default = "default_tools_enabled")]
    pub enabled: bool,

    /// Override specific tools (true to enable, false to disable)
    #[serde(default)]
    pub overrides: HashMap<String, bool>,
}

fn default_tools_enabled() -> bool {
    true
}

impl Config {
    /// Create a new default configuration
    pub fn new() -> Self {
        Self {
            llm: LLMSettings::default(),
            providers: Providers {
                anthropic: ProviderSettings {
                    default_model: Some(DEFAULT_ANTHROPIC_MODEL.to_string()),
                    ..Default::default()
                },
                openai: ProviderSettings {
                    default_model: Some(DEFAULT_OPENAI_MODEL.to_string()),
                    ..Default::default()
                },
                groq: ProviderSettings {
                    default_model: Some(DEFAULT_GROQ_MODEL.to_string()),
                    ..Default::default()
                }
            },
            tools: ToolsConfig::default(),
        }
    }

    /// Load configuration from files
    pub fn load() -> Result<Self> {
        // Start with default configuration
        let mut config = Self::new();

        // Try to load from user config directory
        if let Some(user_config_path) = get_user_config_path() {
            if user_config_path.exists() {
                merge_config_from_file(&mut config, &user_config_path)?;
            }
        }

        // Try to load from project directory (only if we're in a project)
        if let Some(project_config_path) = find_project_config() {
            merge_config_from_file(&mut config, &project_config_path)?;
        }

        Ok(config)
    }

    /// Get the model to use for a provider
    pub fn get_model_for_provider(&self, provider: &str) -> String {
        // First check if there's a default model configured at the top level
        if let Some(model) = &self.llm.default_model {
            return model.clone();
        }

        // Then check for provider-specific default
        match provider {
            "anthropic" => self
                .providers
                .anthropic
                .default_model
                .clone()
                .unwrap_or_else(|| DEFAULT_ANTHROPIC_MODEL.to_string()),
            "openai" => self
                .providers
                .openai
                .default_model
                .clone()
                .unwrap_or_else(|| DEFAULT_OPENAI_MODEL.to_string()),
            "groq" => self
                .providers
                .groq
                .default_model
                .clone()
                .unwrap_or_else(|| DEFAULT_GROQ_MODEL.to_string()),
            _ => DEFAULT_ANTHROPIC_MODEL.to_string(),
        }
    }

    /// Get the API key for a provider, attempting to resolve environment variables
    pub fn get_api_key_for_provider(&self, provider: &str) -> Result<SecretString> {
        let key_value = match provider {
            "anthropic" => {
                // Try config first
                if let Some(key) = &self.providers.anthropic.api_key {
                    expand_env_vars(key)
                } else {
                    // Fall back to environment variable
                    env::var("ANTHROPIC_API_KEY").map_err(|_| {
                        anyhow::anyhow!(
                            "ANTHROPIC_API_KEY environment variable not set and not configured"
                        )
                    })?
                }
            }
            "openai" => {
                // Try config first
                if let Some(key) = &self.providers.openai.api_key {
                    expand_env_vars(key)
                } else {
                    // Fall back to environment variable
                    env::var("OPENAI_API_KEY").map_err(|_| {
                        anyhow::anyhow!(
                            "OPENAI_API_KEY environment variable not set and not configured"
                        )
                    })?
                }
            },
            "groq" => {
                if let Some(key) = &self.providers.groq.api_key {
                    expand_env_vars(key)
                } else {
                    // Fall back to environment variable
                    env::var("OPENAI_API_KEY").map_err(|_| {
                        anyhow::anyhow!(
                            "OPENAI_API_KEY environment variable not set and not configured"
                        )
                    })?
                }
            }
            _ => return Err(anyhow::anyhow!("Unknown provider: {}", provider)),
        };

        Ok(SecretString::new(key_value.into()))
    }

    /// Check if a tool is enabled based on configuration
    pub fn is_tool_enabled(&self, tool_name: &str) -> bool {
        // First check if we have a specific override for this tool
        if let Some(enabled) = self.tools.overrides.get(tool_name) {
            return *enabled;
        }

        // If not, use the global setting
        self.tools.enabled
    }
}

impl ToolsConfig {
    /// Get a list of all enabled tool names based on current configuration and available tools
    pub fn get_enabled_tools(&self, all_tools: &[String]) -> Vec<String> {
        all_tools
            .iter()
            .filter(|tool_name| {
                // If we have a specific override for this tool, use that
                if let Some(enabled) = self.overrides.get(*tool_name) {
                    *enabled
                } else {
                    // Otherwise use the global setting
                    self.enabled
                }
            })
            .cloned()
            .collect()
    }
}

/// Get the path to the user's configuration file
fn get_user_config_path() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("it.nixcode", "nixcode", "nixcode-ai") {
        let config_dir = proj_dirs.config_dir();
        Some(config_dir.join("config.toml"))
    } else {
        None
    }
}

/// Find the project configuration file by walking up the directory tree
fn find_project_config() -> Option<PathBuf> {
    let mut current_dir = env::current_dir().ok()?;

    loop {
        let config_path = current_dir.join(".nixcode").join("config.toml");
        if config_path.exists() {
            return Some(config_path);
        }

        if !current_dir.pop() {
            break;
        }
    }

    None
}

/// Merge configuration from a file into the existing configuration
fn merge_config_from_file(config: &mut Config, path: &Path) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let file_config: Config = toml::from_str(&content)?;

    // For now, we just completely override with the file config
    // In a more complex implementation, we would need to do a deep merge
    *config = file_config;

    Ok(())
}

/// Expand environment variables in a string (format: ${VAR_NAME})
fn expand_env_vars(input: &str) -> String {
    let mut result = input.to_string();

    // Find all patterns like ${VAR_NAME}
    let mut start_idx = 0;
    while let Some(var_start) = result[start_idx..].find("${") {
        let var_start = start_idx + var_start;

        if let Some(var_end) = result[var_start..].find("}") {
            let var_end = var_start + var_end + 1;
            let var_name = &result[var_start + 2..var_end - 1];

            // Replace with environment variable value if it exists
            if let Ok(var_value) = env::var(var_name) {
                result.replace_range(var_start..var_end, &var_value);
                // Continue from the position after the replacement
                start_idx = var_start + var_value.len();
            } else {
                // If environment variable doesn't exist, leave as is and continue
                start_idx = var_end;
            }
        } else {
            // No closing brace found, exit the loop
            break;
        }
    }

    result
}
