# Nixcode Library Documentation

## Overview

The `nixcode` library is a core component of the nixcode-ai project, providing essential functionality for the terminal-based LLM client. This library implements the core business logic, event handling, tool execution, configuration management, and project context handling.

## Table of Contents

1. [Architecture](./architecture.md)
2. [Core Components](./core/README.md)
   - [Nixcode](./core/nixcode.md)
   - [Events](./core/events.md)
   - [Config](./core/config.md)
   - [Project](./core/project.md)
   - [Prompts](./core/prompts.md)
3. [Tools](./tools/README.md)
   - [File System Tools](./tools/fs.md)
   - [Git Tools](./tools/git.md)
   - [GitHub Tools](./tools/github.md)
   - [Search Tools](./tools/search.md)
   - [Glob Tools](./tools/glob.md)
   - [Command Tools](./tools/commands.md)
   - [Prompt Tools](./tools/prompt.md)
4. [Utilities](./utils/README.md)
   - [File System Utilities](./utils/fs.md)
5. [API Reference](./api/README.md)
   - [Public API](./api/public.md)
   - [Tool Trait](./api/tool.md)

## Getting Started

The `nixcode` library is designed to be used as a dependency by the main nixcode-ai application. It provides a high-level API for interacting with LLMs, managing configuration, and executing tools.

```rust
use nixcode::{Nixcode, Project, Config};

// Create a new project
let project = Project::new(std::env::current_dir().unwrap());

// Initialize Nixcode with default configuration
let (rx, nixcode) = Nixcode::new_from_env(project).unwrap();

// Use the Nixcode instance to interact with LLMs
// ...
```

## Key Features

- **Event-driven architecture**: Communication between components via a standardized event system
- **Tool execution framework**: Extensible system for LLM function calling
- **Configuration management**: Layered configuration system with global and project-specific settings
- **Project context**: Maintains information about the current project, including git repository details
- **LLM integration**: Seamless interaction with various LLM providers

## License

This library is part of the nixcode-ai project and is subject to its licensing terms.