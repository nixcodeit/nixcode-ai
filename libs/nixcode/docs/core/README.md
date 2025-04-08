# Core Components

The core components of the `nixcode` library provide the fundamental functionality for the application. These components handle the main business logic, event management, configuration, and project context.

## Components

- [Nixcode](./nixcode.md): The main entry point for the library, handling LLM interactions and tool execution
- [Events](./events.md): The event system for communication between components
- [Config](./config.md): Configuration management for the application
- [Project](./project.md): Project context and metadata
- [Prompts](./prompts.md): System prompts and templates for LLM interactions

## Interactions

The core components interact with each other to provide a cohesive system:

1. **Nixcode** uses **Config** to determine which LLM provider to use and which tools to enable
2. **Nixcode** uses **Project** to get context about the current project
3. **Nixcode** uses **Prompts** to provide system instructions to the LLM
4. **Nixcode** dispatches **Events** to communicate with other components

## Initialization Flow

```rust
// Create a new project
let project = Project::new(std::env::current_dir().unwrap());

// Load configuration
let config = Config::load().unwrap_or_else(|_| Config::new());

// Initialize Nixcode
let (rx, nixcode) = Nixcode::new_with_config(project, config)?;
```

## Event Flow

```
Nixcode → Events → Application
```

Events are dispatched through a channel system, allowing for asynchronous communication between components.

## Configuration Flow

```
Default Config → Global Config → Project Config → Runtime Config
```

Configuration is loaded from multiple sources and merged to create the final configuration used by the application.