# Client API Documentation

The Client module provides implementations for interacting with different LLM providers. It abstracts away the differences between provider APIs, offering a unified interface for sending requests and processing responses.

## Table of Contents

1. [Overview](#overview)
2. [LLMClient Enum](#llmclient-enum)
3. [LLMClientImpl Trait](#llmclientimpl-trait)
4. [Provider Implementations](#provider-implementations)
   - [Anthropic Client](#anthropic-client)
   - [OpenAI Client](#openai-client)
5. [Request Handling](#request-handling)
6. [Stream Processing](#stream-processing)

## Overview

The client module is organized as follows:

```
client/
├── anthropic/
│   ├── client.rs
│   ├── mod.rs
│   ├── request.rs
│   └── stream/
│       ├── error_handler.rs
│       ├── mod.rs
│       └── processor.rs
├── openai/
│   ├── client.rs
│   ├── mod.rs
│   ├── request.rs
│   └── stream/
│       ├── error_handler.rs
│       ├── mod.rs
│       └── processor.rs
├── request/
│   └── mod.rs
└── mod.rs
```

Each provider has its own module with dedicated implementations for client functionality, request formatting, and stream processing.

## LLMClient Enum

The `LLMClient` enum is the main entry point for interacting with LLM providers. It wraps provider-specific client implementations and provides a unified interface.

```rust
pub enum LLMClient {
    OpenAI(OpenAIClient),
    Anthropic(AnthropicClient),
}
```

### Methods

- `new_openai(options: HttpClientOptions) -> Result<Self, LLMError>`: Creates a new OpenAI client
- `new_anthropic(options: HttpClientOptions) -> Result<Self, LLMError>`: Creates a new Anthropic client
- `count_tokens(request: LLMRequest) -> Result<u32, LLMError>`: Counts tokens for a request
- `send(request: LLMRequest) -> Result<UnboundedReceiver<LLMEvent>, LLMError>`: Sends a request and returns a receiver for events

## LLMClientImpl Trait

The `LLMClientImpl` trait defines the interface that all provider implementations must implement:

```rust
pub trait LLMClientImpl {
    fn count_tokens(
        &self,
        request: LLMRequest,
    ) -> impl std::future::Future<Output = Result<u32, LLMError>> + Sync;
    
    fn send(
        &self,
        request: LLMRequest,
    ) -> impl std::future::Future<Output = Result<UnboundedReceiver<LLMEvent>, LLMError>> + Sync;

    fn get_config(&self) -> HttpClientOptions;
}
```

This trait ensures that all provider implementations provide consistent functionality.

## Provider Implementations

### Anthropic Client

The `AnthropicClient` implements the `LLMClientImpl` trait for Anthropic's API:

```rust
pub struct AnthropicClient {
    total_usages: AnthropicUsage,
    history: Vec<Message>,
    client: reqwest::Client,
    config: HttpClientOptions,
}
```

#### Key Methods

- `new(options: HttpClientOptions) -> Result<Self, LLMError>`: Creates a new Anthropic client
- `get_usage() -> AnthropicUsage`: Returns usage statistics
- `get_messages() -> Vec<Message>`: Returns message history

#### Implementation Details

The Anthropic client:
1. Sets up appropriate headers for authentication
2. Formats requests according to Anthropic's API
3. Handles streaming responses
4. Processes token counting using Anthropic's dedicated endpoint

### OpenAI Client

The `OpenAIClient` implements the `LLMClientImpl` trait for OpenAI's API:

```rust
pub struct OpenAIClient {
    total_usages: AnthropicUsage,
    history: Vec<Message>,
    client: reqwest::Client,
    config: HttpClientOptions,
}
```

#### Key Methods

- `new(options: HttpClientOptions) -> Result<Self, LLMError>`: Creates a new OpenAI client
- `get_usage() -> AnthropicUsage`: Returns usage statistics
- `get_messages() -> Vec<Message>`: Returns message history

#### Implementation Details

The OpenAI client:
1. Sets up appropriate headers for authentication
2. Formats requests according to OpenAI's API
3. Handles streaming responses
4. Estimates token counts (as OpenAI doesn't have a dedicated token counting endpoint)

## Request Handling

Each provider module includes a `request.rs` file that handles request formatting:

### Anthropic Request

The `prepare_request_body` function converts a generic `LLMRequest` to Anthropic's format:

```rust
pub fn prepare_request_body(request: &LLMRequest) -> Result<Value, LLMError> {
    // Convert request to Anthropic format
}
```

### OpenAI Request

The `request_to_openai` function converts a generic `LLMRequest` to OpenAI's format:

```rust
pub fn request_to_openai(request: &LLMRequest) -> OpenAIRequest {
    // Convert request to OpenAI format
}
```

## Stream Processing

Each provider module includes a `stream` directory with modules for processing streaming responses:

### Stream Processor

The `process_stream` function processes a streaming response and returns a channel receiver:

```rust
pub async fn process_stream(
    model: LLMModel,
    response: Response,
) -> UnboundedReceiver<LLMEvent> {
    // Process streaming response
}
```

### Error Handler

The `error_handler` module handles errors in the streaming response:

```rust
pub fn handle_error(error: Error) -> LLMEvent {
    // Convert error to LLMEvent
}
```

## Usage Example

Here's a complete example of using the client API:

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::message::common::llm_message::{LLMMessage, LLMRequest};
use llm_sdk::models::llm_model::Sonnet37;
use secrecy::SecretString;
use tokio::runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a runtime for async operations
    let rt = Runtime::new()?;
    
    // Run the async code
    rt.block_on(async {
        // Create a client
        let api_key = SecretString::new("your-api-key".to_string());
        let options = HttpClientOptions::new_anthropic(api_key);
        let client = LLMClient::new_anthropic(options)?;
        
        // Create a request
        let request = LLMRequest {
            model: Sonnet37.clone(),
            messages: vec![
                LLMMessage {
                    role: "user".to_string(),
                    text: Some("Hello, world!".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        
        // Count tokens
        let token_count = client.count_tokens(request.clone()).await?;
        println!("Token count: {}", token_count);
        
        // Send the request
        let mut receiver = client.send(request).await?;
        
        // Process the response
        while let Some(event) = receiver.recv().await {
            match event {
                LLMEvent::PartialContent(_, content) => {
                    println!("Partial content: {:?}", content);
                },
                LLMEvent::Content(content) => {
                    println!("Content: {:?}", content);
                },
                LLMEvent::Stop(reason) => {
                    println!("Stop: {:?}", reason);
                    break;
                },
            }
        }
        
        Ok(())
    })
}
```