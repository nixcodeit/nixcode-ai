# Nixcode

The `Nixcode` struct is the main entry point for the library, providing methods for interacting with LLMs, executing tools, and managing message history.

## Overview

`Nixcode` encapsulates the core functionality of the nixcode-ai application, including:
- LLM communication
- Tool execution
- Message history management
- Event dispatching
- Configuration management
- Project context

## Structure

```rust
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
```

### Fields

- `project`: The project context, containing information about the current project
- `client`: The LLM client for communicating with the LLM provider
- `model`: The LLM model to use for requests
- `tools`: The collection of tools available for execution
- `config`: The application configuration
- `messages`: The message history for the current conversation
- `usage`: Token usage information for the current conversation
- `llm_error`: The most recent error from the LLM, if any
- `is_waiting`: Whether the application is currently waiting for a response from the LLM
- `tx`: The channel for dispatching events

## Initialization

`Nixcode` provides several methods for initialization:

### `new`

```rust
pub fn new(
    project: Project,
    client: LLMClient,
    config: Config,
) -> Result<NewNixcodeResult, LLMError>
```

Creates a new `Nixcode` instance with the provided project, client, and configuration.

### `new_from_env`

```rust
pub fn new_from_env(project: Project) -> Result<NewNixcodeResult, LLMError>
```

Creates a new `Nixcode` instance with configuration loaded from files or environment variables.

### `new_with_config`

```rust
pub fn new_with_config(
    mut project: Project,
    config: Config,
) -> Result<NewNixcodeResult, LLMError>
```

Creates a new `Nixcode` instance with the provided configuration.

## LLM Interaction

### `send`

```rust
pub async fn send(self: Arc<Self>, messages: Vec<LLMMessage>)
```

Sends the provided messages to the LLM and processes the response.

### `send_message`

```rust
pub async fn send_message(self: Arc<Self>, message: Option<LLMMessage>)
```

Adds the provided message to the history and sends all messages to the LLM.

### `handle_response_event`

```rust
pub async fn handle_response_event(self: &Arc<Self>, message: LLMEvent)
```

Processes an event from the LLM response stream.

## Tool Execution

### `execute_tools`

```rust
pub async fn execute_tools(self: &Arc<Self>)
```

Executes any tools requested by the LLM in the most recent response.

### `send_tools_results`

```rust
pub async fn send_tools_results(self: Arc<Self>)
```

Sends the results of tool execution back to the LLM.

## Message Management

### `add_message`

```rust
async fn add_message(&self, message: LLMMessage)
```

Adds a message to the history and dispatches a `NewMessage` event.

### `remove_last_message`

```rust
pub async fn remove_last_message(self: &Arc<Self>)
```

Removes the last message from the history.

### `reset`

```rust
pub async fn reset(self: &Arc<Self>) -> Result<()>
```

Clears the message history and resets usage statistics.

### `retry_last_message`

```rust
pub async fn retry_last_message(self: &Arc<Self>)
```

Removes the last assistant message and resends the conversation to the LLM.

## State Management

### `is_waiting`

```rust
pub async fn is_waiting(&self) -> bool
```

Returns whether the application is currently waiting for a response from the LLM.

### `set_waiting`

```rust
pub async fn set_waiting(&self, new_val: bool)
```

Sets the waiting state of the application.

## Accessors

### `get_config`

```rust
pub fn get_config(&self) -> &Config
```

Returns the current configuration.

### `get_model`

```rust
pub fn get_model(&self) -> &'static LLMModel
```

Returns the current LLM model.

### `get_project`

```rust
pub fn get_project(&self) -> Arc<Project>
```

Returns the current project context.

### `get_messages`

```rust
pub async fn get_messages(&self) -> Vec<LLMMessage>
```

Returns the current message history.

### `get_error`

```rust
pub async fn get_error(&self) -> Option<ErrorEventContent>
```

Returns the most recent error from the LLM, if any.

### `get_usage`

```rust
pub async fn get_usage(&self) -> AnthropicUsage
```

Returns the token usage information for the current conversation.

### `has_init_analysis`

```rust
pub fn has_init_analysis(&self) -> bool
```

Returns whether the project has an initialization analysis.

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