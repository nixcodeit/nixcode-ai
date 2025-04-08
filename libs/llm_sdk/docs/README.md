# LLM SDK Documentation

The LLM SDK is a Rust library that provides a unified interface for interacting with various Large Language Model (LLM) providers. This SDK abstracts away the differences between different LLM APIs, allowing developers to easily switch between providers while maintaining a consistent interface.

## Table of Contents

1. [Overview](#overview)
2. [Supported Providers](#supported-providers)
3. [Architecture](#architecture)
4. [Getting Started](#getting-started)
5. [Core Concepts](#core-concepts)
6. [API Reference](#api-reference)
7. [Advanced Usage](#advanced-usage)

## Overview

The LLM SDK is designed to provide a consistent interface for interacting with different LLM providers. It handles:

- Authentication and API key management
- Request formatting for different providers
- Response parsing and streaming
- Error handling
- Token counting and cost calculation
- Tool/function calling support
- Message history management

The SDK is built with a modular architecture that makes it easy to add support for new providers and models.

## Supported Providers

The SDK currently supports the following LLM providers:

- **Anthropic**: Claude 3.7 Sonnet, Claude 3.5 Haiku
- **OpenAI**: GPT-4o, GPT-3o Mini
- **Groq**: DeepSeek R1, Llama 4 Scout, Qwen Qwq 32b
- **OpenRouter**: Quasar Alpha, DeepSeek V3, Llama 4 Scout, Gemini 2.5 Pro Preview
- **Gemini**: Google's Gemini models

## Architecture

The LLM SDK follows a modular architecture with the following key components:

### Client Module

The `client` module provides implementations for different LLM providers:

- `anthropic`: Anthropic API client implementation
- `openai`: OpenAI API client implementation
- `request`: Common request handling utilities

Each provider implementation includes:
- Client: Handles authentication and API communication
- Request: Formats requests according to the provider's API
- Stream: Processes streaming responses

### Models Module

The `models` module defines the supported LLM models and their capabilities:

- `llm_model.rs`: Defines the `LLMModel` struct and builder pattern
- `capabilities.rs`: Defines model capabilities (streaming, cache, thinking)
- Provider-specific model implementations (e.g., `anthropic/sonnet37.rs`)

### Message Module

The `message` module handles message formatting and processing:

- `message.rs`: Defines the `Message` enum for user and assistant messages
- `content`: Defines different content types (text, image, tools, thinking)
- Provider-specific message implementations

### Tools Module

The `tools` module provides support for function calling:

- `tools.rs`: Defines the `Tool` struct for function definitions
- `json_schema.rs`: Handles JSON schema for tool parameters

### Configuration

The `config` module handles client configuration:

- `HttpClientOptions`: Configuration for HTTP clients (API keys, base URLs)

### Error Handling

The `errors` module provides standardized error handling:

- `LLMError`: Enum for different error types
- Conversion to provider-specific error formats

## Getting Started

To use the LLM SDK, you need to:

1. Create a client for your preferred provider
2. Prepare a request with your messages
3. Send the request and process the response

Here's a basic example:

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::message::common::llm_message::LLMRequest;
use llm_sdk::models::llm_model::Sonnet37;
use secrecy::SecretString;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let api_key = SecretString::new("your-api-key".to_string());
    let options = HttpClientOptions::new_anthropic(api_key);
    let client = LLMClient::new_anthropic(options)?;
    
    // Prepare a request
    let request = LLMRequest {
        model: Sonnet37.clone(),
        messages: vec![
            // Add your messages here
        ],
        ..Default::default()
    };
    
    // Send the request
    let mut receiver = client.send(request).await?;
    
    // Process the response
    while let Some(event) = receiver.recv().await {
        // Handle the event
        println!("{:?}", event);
    }
    
    Ok(())
}
```

## Core Concepts

### LLM Models

The SDK uses a builder pattern to define LLM models with their capabilities:

```rust
pub static ref Sonnet37: LLMModel = LLMModelBuilder::new()
    .model_name("claude-3-7-sonnet-latest")
    .display_name("Claude 3.7 Sonnet")
    .provider(LLMProvider::Anthropic)
    .capabilities(
        ModelCapabilitiesBuilder::new()
            .with_cache()
            .with_streaming()
            .build()
    )
    .cost_calculation(Arc::new(sonnet37_cost_calculation))
    .build();
```

### Messages

Messages are represented using the `Message` enum:

```rust
pub enum Message {
    User(Contents),
    Assistant(Contents),
}
```

Where `Contents` is a vector of `Content` items, which can be:

- `Text`: Plain text content
- `Image`: Image content
- `Thinking`: Reasoning/thinking content
- `ToolUse`: Tool/function call
- `ToolResult`: Result of a tool/function call

### Events

The SDK uses an event-based system for streaming responses:

```rust
pub enum LLMEvent {
    PartialContent(usize, Content),
    Content(Content),
    Stop(Option<StopReason>),
}
```

### Cost Calculation

The SDK includes cost calculation functions for different models:

```rust
fn sonnet37_cost_calculation(usage: Usage) -> f64 {
    // Calculate cost based on input and output tokens
}
```

## API Reference

For detailed API documentation, please refer to the following sections:

- [Client API](./client.md)
- [Models API](./models.md)
- [Message API](./message.md)
- [Tools API](./tools.md)
- [Configuration API](./config.md)
- [Error Handling](./errors.md)

## Advanced Usage

For advanced usage examples, please refer to the following sections:

- [Streaming Responses](./streaming.md)
- [Tool/Function Calling](./tools_usage.md)
- [Token Counting](./tokens.md)
- [Cost Calculation](./cost.md)
- [Custom Providers](./custom_providers.md)