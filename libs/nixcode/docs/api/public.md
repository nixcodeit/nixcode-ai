# Public API

The public API of the nixcode library provides the main entry points for interacting with the library.

## Overview

The public API consists of the following main components:
- `Nixcode`: The main entry point for the library
- `Project`: Represents the context of the current project
- `Config`: Represents the application configuration
- `NixcodeEvent`: Represents events dispatched by the library

## Nixcode

The `Nixcode` struct is the main entry point for the library, providing methods for interacting with LLMs, executing tools, and managing message history.

### Initialization

```rust
pub fn new(
    project: Project,
    client: LLMClient,
    config: Config,
) -> Result<NewNixcodeResult, LLMError>
```

Creates a new `Nixcode` instance with the provided project, client, and configuration.

```rust
pub fn new_from_env(project: Project) -> Result<NewNixcodeResult, LLMError>
```

Creates a new `Nixcode` instance with configuration loaded from files or environment variables.

```rust
pub fn new_with_config(
    mut project: Project,
    config: Config,
) -> Result<NewNixcodeResult, LLMError>
```

Creates a new `Nixcode` instance with the provided configuration.

### LLM Interaction

```rust
pub async fn send(self: Arc<Self>, messages: Vec<LLMMessage>)
```

Sends the provided messages to the LLM and processes the response.

```rust
pub async fn send_message(self: Arc<Self>, message: Option<LLMMessage>)
```

Adds the provided message to the history and sends all messages to the LLM.

### Tool Execution

```rust
pub async fn execute_tools(self: &Arc<Self>)
```

Executes any tools requested by the LLM in the most recent response.

```rust
pub async fn send_tools_results(self: Arc<Self>)
```

Sends the results of tool execution back to the LLM.

### Message Management

```rust
pub async fn remove_last_message(self: &Arc<Self>)
```

Removes the last message from the history.

```rust
pub async fn reset(self: &Arc<Self>) -> Result<()>
```

Clears the message history and resets usage statistics.

```rust
pub async fn retry_last_message(self: &Arc<Self>)
```

Removes the last assistant message and resends the conversation to the LLM.

### State Management

```rust
pub async fn is_waiting(&self) -> bool
```

Returns whether the application is currently waiting for a response from the LLM.

```rust
pub async fn set_waiting(&self, new_val: bool)
```

Sets the waiting state of the application.

### Accessors

```rust
pub fn get_config(&self) -> &Config
```

Returns the current configuration.

```rust
pub fn get_model(&self) -> &'static LLMModel
```

Returns the current LLM model.

```rust
pub fn get_project(&self) -> Arc<Project>
```

Returns the current project context.

```rust
pub async fn get_messages(&self) -> Vec<LLMMessage>
```

Returns the current message history.

```rust
pub async fn get_error(&self) -> Option<ErrorEventContent>
```

Returns the most recent error from the LLM, if any.

```rust
pub async fn get_usage(&self) -> AnthropicUsage
```

Returns the token usage information for the current conversation.

```rust
pub fn has_init_analysis(&self) -> bool
```

Returns whether the project has an initialization analysis.

## Project

The `Project` struct represents the context of the current project, including the current working directory, git repository information, and project analysis.

### Initialization

```rust
pub fn new(cwd: PathBuf) -> Self
```

Creates a new `Project` instance with the provided current working directory.

### Accessors

```rust
pub fn get_cwd(&self) -> PathBuf
```

Returns the current working directory.

```rust
pub fn get_project_init_analysis_content(&self) -> Option<String>
```

Returns the content of the project initialization analysis, if available.

```rust
pub fn has_init_analysis(&self) -> bool
```

Returns whether the project has an initialization analysis.

```rust
pub fn has_repo_path(&self) -> bool
```

Returns whether the project has a git repository.

```rust
pub fn get_repo_path(&self) -> Option<PathBuf>
```

Returns the path to the git repository, if available.

### GitHub Integration

```rust
pub fn set_github(&mut self, github: &GitHubSettings) -> &mut Self
```

Sets the GitHub repository information from the provided settings.

```rust
pub fn get_github(&self) -> Option<GitHub>
```

Returns the GitHub repository information, if available.

## Config

The `Config` struct represents the application configuration, including LLM provider settings, tool configuration, and GitHub integration.

### Initialization

```rust
pub fn new() -> Self
```

Creates a new default configuration.

```rust
pub fn load() -> Result<Self>
```

Loads configuration from files, merging global and project-specific settings.

### Accessors

```rust
pub fn get_api_key_for_provider(&self, provider: &str) -> Result<SecretString>
```

Gets the API key for a provider, attempting to resolve environment variables.

```rust
pub fn get_github_token(&self) -> Result<String>
```

Gets the GitHub token, attempting to resolve environment variables.

```rust
pub fn get_model_for_provider(&self, provider: &str) -> &'static LLMModel
```

Gets the default model for a provider.

```rust
pub fn is_tool_enabled(&self, tool_name: &str) -> bool
```

Checks if a tool is enabled based on configuration.

## NixcodeEvent

The `NixcodeEvent` enum represents events dispatched by the library.

```rust
pub enum NixcodeEvent {
    GeneratingResponse,
    GeneratedResponse,
    NewMessage,
    MessageUpdated,
    Error(LLMError),
    ToolStart(ToolCall),
    ToolEnd(ToolResult),
    ToolsFinished,
}
```

### Event Types

- `GeneratingResponse`: Indicates that the application is generating a response from the LLM
- `GeneratedResponse`: Indicates that the application has finished generating a response from the LLM
- `NewMessage`: Indicates that a new message has been added to the conversation
- `MessageUpdated`: Indicates that a message has been updated (typically during streaming)
- `Error(LLMError)`: Indicates that an error has occurred during LLM communication
- `ToolStart(ToolCall)`: Indicates that a tool execution has started
- `ToolEnd(ToolResult)`: Indicates that a tool execution has completed
- `ToolsFinished`: Indicates that all tool executions have completed

## Example Usage

```rust
// Create a new project
let project = Project::new(std::env::current_dir().unwrap());

// Initialize Nixcode with default configuration
let (rx, nixcode) = Nixcode::new_from_env(project).unwrap();
let nixcode = Arc::new(nixcode);

// Send a message to the LLM
let message = LLMMessage::new_user("Hello, world!");
nixcode.clone().send_message(Some(message)).await;

// Process events from the LLM
while let Some(event) = rx.recv().await {
    match event {
        NixcodeEvent::NewMessage => {
            // Handle new message
        }
        NixcodeEvent::MessageUpdated => {
            // Handle message update
        }
        NixcodeEvent::ToolsFinished => {
            // Handle tool execution completion
            nixcode.clone().send_tools_results().await;
        }
        // Handle other events
        _ => {}
    }
}
```