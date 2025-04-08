# Configuration

The configuration module provides a flexible system for managing application settings, including LLM provider settings, tool configuration, and GitHub integration.

## Overview

The configuration system uses a layered approach:
1. Default configuration (hardcoded)
2. Global configuration (`~/.config/nixcode-ai/config.toml`)
3. Project configuration (`.nixcode/config.toml`)

Configuration is loaded from these sources and merged to create the final configuration used by the application.

## Config Structure

The `Config` struct represents the application configuration:

```rust
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

    /// GitHub configuration
    #[serde(default)]
    pub github: GitHubSettings,
}
```

### LLM Settings

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LLMSettings {
    /// Default LLM provider to use
    #[serde(default = "default_provider")]
    pub default_provider: String,

    #[serde(default)]
    pub overrides: HashMap<String, bool>,
}
```

### Provider Settings

```rust
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

    /// OpenRouter-specific settings
    #[serde(default)]
    pub open_router: ProviderSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProviderSettings {
    /// API key for the provider (can use ${ENV_VAR} syntax)
    pub api_key: Option<String>,
}
```

### Tool Configuration

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ToolsConfig {
    /// Enable all tools by default
    #[serde(default = "default_tools_enabled")]
    pub enabled: bool,

    /// Override specific tools (true to enable, false to disable)
    #[serde(default)]
    pub overrides: HashMap<String, bool>,
}
```

### GitHub Settings

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GitHubSettings {
    /// GitHub API token (can use ${ENV_VAR} syntax)
    pub token: Option<String>,

    /// GitHub profile/organization name
    pub org: Option<String>,

    /// GitHub repository name
    pub repo: Option<String>,
}
```

## Configuration Methods

### Loading Configuration

```rust
pub fn load() -> Result<Self>
```

Loads configuration from files, merging global and project-specific settings.

### Getting API Keys

```rust
pub fn get_api_key_for_provider(&self, provider: &str) -> Result<SecretString>
```

Gets the API key for a provider, attempting to resolve environment variables.

```rust
pub fn get_github_token(&self) -> Result<String>
```

Gets the GitHub token, attempting to resolve environment variables.

### Getting Model Information

```rust
pub fn get_model_for_provider(&self, provider: &str) -> &'static LLMModel
```

Gets the default model for a provider.

### Tool Configuration

```rust
pub fn is_tool_enabled(&self, tool_name: &str) -> bool
```

Checks if a tool is enabled based on configuration.

## Configuration File Format

Configuration files use the TOML format:

```toml
[llm]
default_provider = "anthropic"

[providers.anthropic]
api_key = "${ANTHROPIC_API_KEY}"

[providers.openai]
api_key = "${OPENAI_API_KEY}"

[providers.groq]
api_key = "${GROQ_API_KEY}"

[providers.open_router]
api_key = "${OPENROUTER_API_KEY}"

[tools]
enabled = true

[tools.overrides]
git_add = false
read_text_file = true

[github]
token = "${GITHUB_TOKEN}"
org = "nixcode-ai"
repo = "nixcode-ai"
```

## Environment Variable Expansion

The configuration system supports environment variable expansion in string values using the `${VAR_NAME}` syntax:

```toml
[providers.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
```

This will be expanded to the value of the `ANTHROPIC_API_KEY` environment variable.

## Configuration Paths

- Global configuration: `~/.config/nixcode-ai/config.toml`
- Project configuration: `.nixcode/config.toml`

## Example Usage

```rust
// Load configuration
let config = Config::load().unwrap_or_else(|_| Config::new());

// Get API key for provider
let api_key = config.get_api_key_for_provider("anthropic").unwrap();

// Check if tool is enabled
if config.is_tool_enabled("git_add") {
    // Use git_add tool
}

// Get default model for provider
let model = config.get_model_for_provider("anthropic");
```