# API Reference

The API reference provides detailed documentation for the public API of the nixcode library.

## Overview

The nixcode library provides a high-level API for interacting with LLMs, managing configuration, and executing tools. This API is designed to be used by the main nixcode-ai application.

## API Categories

- [Public API](./public.md): The main public API for the nixcode library
- [Tool Trait](./tool.md): The trait that all tools must implement

## Usage

The nixcode library is typically used by the main nixcode-ai application:

```rust
use nixcode::{Nixcode, Project, Config};

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