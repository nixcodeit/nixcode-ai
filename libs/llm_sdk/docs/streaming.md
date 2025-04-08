# Streaming Responses Documentation

This document explains how to work with streaming responses in the LLM SDK. Streaming allows you to receive and process partial responses from LLM providers as they are generated, rather than waiting for the complete response.

## Table of Contents

1. [Overview](#overview)
2. [Streaming Architecture](#streaming-architecture)
3. [Event Types](#event-types)
4. [Provider-Specific Implementations](#provider-specific-implementations)
5. [Processing Streaming Responses](#processing-streaming-responses)
6. [Usage Examples](#usage-examples)

## Overview

Streaming is a key feature of the LLM SDK that enables:

- Real-time display of LLM responses
- Incremental processing of content
- Improved user experience with faster feedback
- Handling of long responses efficiently

The SDK uses Tokio channels to stream responses from LLM providers to your application.

## Streaming Architecture

The streaming architecture consists of:

1. **HTTP Streaming**: Using Server-Sent Events (SSE) to receive data from the LLM provider
2. **Event Processing**: Converting provider-specific events to a standardized format
3. **Channel Communication**: Using Tokio channels to send events to your application

### Flow Diagram

```
LLM Provider API (SSE) → Stream Processor → Tokio Channel → Your Application
```

## Event Types

The SDK uses the `LLMEvent` enum to represent different types of events in the stream:

```rust
pub enum LLMEvent {
    PartialContent(usize, Content),
    Content(Content),
    Stop(Option<StopReason>),
}
```

### Event Types

- `PartialContent`: Partial content with an index and content
- `Content`: Complete content
- `Stop`: End of stream with an optional stop reason

### Content Types

The `Content` enum represents different types of content:

```rust
pub enum Content {
    Empty,
    Text(TextContent),
    Image(ImageContent),
    Thinking(ThinkingContent),
    RedactedThinking(RedactedThinkingContent),
    ToolUse(ToolUseContent),
    ToolResult(ToolResultContent),
}
```

### Content Delta

The `ContentDelta` enum represents incremental updates to content:

```rust
pub enum ContentDelta {
    TextDelta(ContentTextDelta),
    ThinkingDelta(ContentThinkingDelta),
    SignatureDelta(ContentSignatureDelta),
    InputJsonDelta(ContentInputJsonDelta),
}
```

## Provider-Specific Implementations

Each provider has its own implementation for processing streaming responses:

### Anthropic Streaming

The `anthropic/stream/processor.rs` file handles Anthropic's streaming format:

```rust
pub async fn process_stream(
    model: LLMModel,
    response: Response,
) -> UnboundedReceiver<LLMEvent> {
    // Process Anthropic streaming response
}
```

### OpenAI Streaming

The `openai/stream/processor.rs` file handles OpenAI's streaming format:

```rust
pub async fn process_stream(
    model: LLMModel,
    response: Response,
) -> UnboundedReceiver<LLMEvent> {
    // Process OpenAI streaming response
}
```

## Processing Streaming Responses

To process streaming responses:

1. Send a request to the LLM provider
2. Receive a channel receiver for events
3. Process events as they arrive

```rust
// Send the request
let mut receiver = client.send(request).await?;

// Process events
while let Some(event) = receiver.recv().await {
    match event {
        LLMEvent::PartialContent(index, content) => {
            // Handle partial content
        },
        LLMEvent::Content(content) => {
            // Handle complete content
        },
        LLMEvent::Stop(reason) => {
            // Handle end of stream
            break;
        },
    }
}
```

## Usage Examples

### Basic Streaming Example

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::message::common::llm_message::{LLMEvent, LLMMessage, LLMRequest};
use llm_sdk::message::content::Content;
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
            LLMMessage {
                role: "user".to_string(),
                text: Some("Tell me a story about a robot learning to paint.".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    
    // Send the request
    let mut receiver = client.send(request).await?;
    
    // Process the response
    let mut full_text = String::new();
    
    while let Some(event) = receiver.recv().await {
        match event {
            LLMEvent::PartialContent(_, content) => {
                if let Content::Text(text) = &content {
                    print!("{}", text.text);
                    std::io::stdout().flush()?;
                    full_text.push_str(&text.text);
                }
            },
            LLMEvent::Content(content) => {
                if let Content::Text(text) = &content {
                    println!("\nFinal content: {}", text.text);
                }
            },
            LLMEvent::Stop(reason) => {
                println!("\nStream ended: {:?}", reason);
                break;
            },
        }
    }
    
    println!("\nFull text: {}", full_text);
    
    Ok(())
}
```

### Handling Different Content Types

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::message::common::llm_message::{LLMEvent, LLMMessage, LLMRequest};
use llm_sdk::message::content::Content;
use llm_sdk::models::llm_model::Sonnet37;
use llm_sdk::ThinkingOptions;
use secrecy::SecretString;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let api_key = SecretString::new("your-api-key".to_string());
    let options = HttpClientOptions::new_anthropic(api_key);
    let client = LLMClient::new_anthropic(options)?;
    
    // Prepare a request with thinking enabled
    let request = LLMRequest {
        model: Sonnet37.clone(),
        messages: vec![
            LLMMessage {
                role: "user".to_string(),
                text: Some("Solve this math problem: If x + y = 10 and x - y = 4, what are x and y?".to_string()),
                ..Default::default()
            },
        ],
        thinking: Some(ThinkingOptions::new(1000)),
        ..Default::default()
    };
    
    // Send the request
    let mut receiver = client.send(request).await?;
    
    // Process the response
    while let Some(event) = receiver.recv().await {
        match event {
            LLMEvent::PartialContent(_, content) => {
                match &content {
                    Content::Text(text) => {
                        println!("Text: {}", text.text);
                    },
                    Content::Thinking(thinking) => {
                        println!("Thinking: {}", thinking.thinking);
                    },
                    Content::ToolUse(tool_use) => {
                        println!("Tool use: {} with input: {:?}", tool_use.name, tool_use.input);
                    },
                    _ => {
                        println!("Other content: {:?}", content);
                    }
                }
            },
            LLMEvent::Content(content) => {
                println!("Final content: {:?}", content);
            },
            LLMEvent::Stop(reason) => {
                println!("Stream ended: {:?}", reason);
                break;
            },
        }
    }
    
    Ok(())
}
```

### Building a UI with Streaming

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::message::common::llm_message::{LLMEvent, LLMMessage, LLMRequest};
use llm_sdk::message::content::Content;
use llm_sdk::models::llm_model::Sonnet37;
use secrecy::SecretString;
use tokio::sync::mpsc::{self, Sender};

// UI update event
enum UIEvent {
    AppendText(String),
    SetThinking(bool),
    Complete,
}

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a channel for UI updates
    let (ui_tx, mut ui_rx) = mpsc::channel::<UIEvent>(100);
    
    // Spawn a task to handle UI updates
    tokio::spawn(async move {
        while let Some(event) = ui_rx.recv().await {
            match event {
                UIEvent::AppendText(text) => {
                    // In a real UI, you would update a text widget
                    print!("{}", text);
                    std::io::stdout().flush().unwrap();
                },
                UIEvent::SetThinking(thinking) => {
                    // In a real UI, you would show/hide a thinking indicator
                    if thinking {
                        println!("\n[Thinking...]");
                    } else {
                        println!("\n[Done thinking]");
                    }
                },
                UIEvent::Complete => {
                    // In a real UI, you would update the UI state
                    println!("\n[Complete]");
                    break;
                },
            }
        }
    });
    
    // Create a client
    let api_key = SecretString::new("your-api-key".to_string());
    let options = HttpClientOptions::new_anthropic(api_key);
    let client = LLMClient::new_anthropic(options)?;
    
    // Prepare a request
    let request = LLMRequest {
        model: Sonnet37.clone(),
        messages: vec![
            LLMMessage {
                role: "user".to_string(),
                text: Some("Tell me about the history of artificial intelligence.".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    
    // Send the request and process the response
    process_stream(client, request, ui_tx).await?;
    
    Ok(())
}

async fn process_stream(
    client: LLMClient,
    request: LLMRequest,
    ui_tx: Sender<UIEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Send the request
    let mut receiver = client.send(request).await?;
    
    // Process the response
    while let Some(event) = receiver.recv().await {
        match event {
            LLMEvent::PartialContent(_, content) => {
                match &content {
                    Content::Text(text) => {
                        ui_tx.send(UIEvent::AppendText(text.text.clone())).await?;
                    },
                    Content::Thinking(_) => {
                        ui_tx.send(UIEvent::SetThinking(true)).await?;
                    },
                    _ => {},
                }
            },
            LLMEvent::Content(_) => {
                ui_tx.send(UIEvent::SetThinking(false)).await?;
            },
            LLMEvent::Stop(_) => {
                ui_tx.send(UIEvent::Complete).await?;
                break;
            },
        }
    }
    
    Ok(())
}
```

### Handling Timeouts

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::message::common::llm_message::{LLMEvent, LLMMessage, LLMRequest};
use llm_sdk::models::llm_model::Sonnet37;
use secrecy::SecretString;
use tokio::time::{timeout, Duration};

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let api_key = SecretString::new("your-api-key".to_string());
    let options = HttpClientOptions::new_anthropic(api_key);
    let client = LLMClient::new_anthropic(options)?;
    
    // Prepare a request
    let request = LLMRequest {
        model: Sonnet37.clone(),
        messages: vec![
            LLMMessage {
                role: "user".to_string(),
                text: Some("Tell me a story.".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    
    // Send the request with a timeout
    let receiver_result = timeout(
        Duration::from_secs(30),
        client.send(request)
    ).await;
    
    // Handle timeout
    let mut receiver = match receiver_result {
        Ok(result) => result?,
        Err(_) => {
            println!("Request timed out");
            return Ok(());
        }
    };
    
    // Process the response with a timeout for each event
    while let Ok(event_result) = timeout(
        Duration::from_secs(5),
        receiver.recv()
    ).await {
        match event_result {
            Some(event) => {
                // Process the event
                println!("{:?}", event);
                
                // Check for stop event
                if let LLMEvent::Stop(_) = event {
                    break;
                }
            },
            None => {
                println!("Stream ended");
                break;
            }
        }
    }
    
    Ok(())
}
```