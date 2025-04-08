# Nixcode Library Architecture

## Overview

The `nixcode` library follows a modular architecture designed around an event-driven system. It serves as the core business logic layer for the nixcode-ai application, handling LLM interactions, tool execution, configuration management, and project context.

## Architectural Patterns

### Event-Driven Architecture

The library uses an event-driven architecture where components communicate through a standardized event system. The `NixcodeEvent` enum defines the types of events that can be dispatched, and components use channels to send and receive these events.

```
User Input → App → Nixcode → LLM SDK → LLM Provider
LLM Response → SDK → Nixcode → Events → App → UI
```

### Tool Execution Framework

The library implements a tool execution framework that allows LLMs to invoke functions through a standardized interface. Tools are registered with the system and can be enabled or disabled through configuration.

```
LLM Request → Tool Execution → Results → LLM
```

### Layered Configuration

The configuration system uses a layered approach:
1. Default configuration (hardcoded)
2. Global configuration (`~/.config/nixcode-ai/config.toml`)
3. Project configuration (`.nixcode/config.toml`)

### Modular Organization

The codebase is organized into modules with clear responsibilities:
- `config`: Configuration management
- `events`: Event definitions and handling
- `project`: Project context and metadata
- `prompts`: System prompts and templates
- `tools`: Tool implementations (fs, git, search, etc.)
- `utils`: Utility functions

## Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                         Nixcode                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────────┐   │
│  │   Config    │   │   Project   │   │     Events      │   │
│  └─────────────┘   └─────────────┘   └─────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                       Tools                          │   │
│  │                                                      │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐    │   │
│  │  │    FS   │ │   Git   │ │  Search │ │ Commands│    │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘    │   │
│  │                                                      │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐    │   │
│  │  │  GitHub │ │   Glob  │ │  Prompt │ │  Utils  │    │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘    │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Data Flow

1. **Initialization Flow**:
   - Create a `Project` instance with the current working directory
   - Load configuration from files or environment variables
   - Initialize the `Nixcode` instance with the project and configuration
   - Set up event channels for communication

2. **Message Flow**:
   - User messages are added to the message history
   - Messages are sent to the LLM via the appropriate provider
   - Responses are streamed back and processed as events
   - UI updates based on these events

3. **Tool Execution Flow**:
   - LLM responses may include tool invocation requests
   - Tool requests are identified and executed
   - Results are sent back to the LLM
   - Conversation continues with tool output context

## Key Interfaces

### Nixcode

The `Nixcode` struct is the main entry point for the library, providing methods for:
- Sending messages to LLMs
- Executing tools
- Managing message history
- Handling LLM responses

### Tool Trait

The `Tool` trait defines the interface for all tools:
```rust
#[async_trait]
pub trait Tool {
    fn get_name(&self) -> String;
    fn get_schema(&self) -> nixcode_llm_sdk::tools::Tool;
    async fn execute(
        &self,
        params: serde_json::Value,
        project: Arc<Project>,
    ) -> anyhow::Result<serde_json::Value>;
}
```

### Events

The `NixcodeEvent` enum defines the types of events that can be dispatched:
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

## Design Decisions

1. **Event-Driven Communication**: Allows for loose coupling between components and asynchronous processing of LLM responses.

2. **Tool Registration System**: Enables dynamic registration and configuration of tools, making it easy to add new functionality.

3. **Project Context**: Maintains information about the current project, including git repository details, allowing tools to operate in the correct context.

4. **Configuration Layering**: Provides flexibility in configuration, allowing for global and project-specific settings.

5. **Async/Await**: Extensive use of Rust's async/await for non-blocking operations, particularly important for LLM interactions and tool execution.