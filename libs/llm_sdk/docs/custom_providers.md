# Custom Providers Documentation

This document explains how to implement custom LLM providers in the LLM SDK. This is useful if you want to integrate with a provider that isn't officially supported or if you have a custom LLM deployment.

## Table of Contents

1. [Overview](#overview)
2. [Implementation Steps](#implementation-steps)
3. [Provider Implementation](#provider-implementation)
4. [Client Implementation](#client-implementation)
5. [Request Formatting](#request-formatting)
6. [Stream Processing](#stream-processing)
7. [Model Definition](#model-definition)
8. [Integration](#integration)
9. [Complete Example](#complete-example)

## Overview

The LLM SDK is designed to be extensible, allowing you to add support for custom LLM providers. To implement a custom provider, you need to:

1. Define a new provider in the `LLMProvider` enum
2. Implement a client for the provider
3. Implement request formatting for the provider
4. Implement stream processing for the provider
5. Define models for the provider
6. Integrate the provider with the SDK

## Implementation Steps

### Step 1: Define a New Provider

Add your provider to the `LLMProvider` enum in `providers.rs`:

```rust
pub enum LLMProvider {
    Anthropic,
    OpenAI,
    Gemini,
    Groq,
    OpenRouter,
    MyCustomProvider, // Add your provider here
}

impl LLMProvider {
    pub fn config_key(&self) -> &'static str {
        match self {
            // Existing providers...
            LLMProvider::MyCustomProvider => "my_custom_provider",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            // Existing providers...
            LLMProvider::MyCustomProvider => "MyCustomProvider",
        }
    }

    pub fn default_model(&self) -> &'static LLMModel {
        match self {
            // Existing providers...
            LLMProvider::MyCustomProvider => &MyCustomModel,
            _ => panic!("No default model for provider: {}", self.name()),
        }
    }
}
```

### Step 2: Create a Client Module

Create a new module for your provider in the `client` directory:

```
client/
├── my_custom_provider/
│   ├── client.rs
│   ├── mod.rs
│   ├── request.rs
│   └── stream/
│       ├── error_handler.rs
│       ├── mod.rs
│       └── processor.rs
```

### Step 3: Define Models

Define models for your provider in the `models` directory:

```rust
lazy_static! {
    pub static ref MyCustomModel: LLMModel = LLMModelBuilder::new()
        .model_name("my-custom-model")
        .display_name("My Custom Model")
        .provider(LLMProvider::MyCustomProvider)
        .capabilities(
            ModelCapabilitiesBuilder::new()
                .with_streaming()
                .build()
        )
        .cost_calculation(Arc::new(my_custom_model_cost_calculation))
        .build();
}

fn my_custom_model_cost_calculation(usage: Usage) -> f64 {
    let input_cost = usage.input_tokens as f64 / 1_000_000.0 * 1.00;
    let output_cost = usage.output_tokens as f64 / 1_000_000.0 * 2.00;
    (input_cost + output_cost).max(0.0)
}
```

### Step 4: Update the LLMClient Enum

Update the `LLMClient` enum in `client/mod.rs`:

```rust
pub enum LLMClient {
    OpenAI(OpenAIClient),
    Anthropic(AnthropicClient),
    MyCustomProvider(MyCustomProviderClient), // Add your client here
}

impl LLMClient {
    // Existing methods...

    pub fn new_my_custom_provider(options: HttpClientOptions) -> anyhow::Result<Self, LLMError> {
        let client = MyCustomProviderClient::new(options);

        if let Err(client) = client {
            return Err(client);
        }

        Ok(LLMClient::MyCustomProvider(client?))
    }

    pub async fn count_tokens(&self, request: LLMRequest) -> Result<u32, LLMError> {
        match self {
            // Existing providers...
            LLMClient::MyCustomProvider(client) => client.count_tokens(request).await,
        }
    }

    pub async fn send(&self, request: LLMRequest) -> Result<UnboundedReceiver<LLMEvent>, LLMError> {
        match self {
            // Existing providers...
            LLMClient::MyCustomProvider(client) => client.send(request).await,
        }
    }
}
```

### Step 5: Add Configuration

Add configuration for your provider in `config.rs`:

```rust
impl HttpClientOptions {
    // Existing methods...

    pub fn new_my_custom_provider(api_key: SecretString) -> Self {
        Self {
            provider: LLMProvider::MyCustomProvider,
            api_key,
            api_base: Some("https://api.mycustomprovider.com".to_string()),
        }
    }
}
```

## Provider Implementation

### Client Implementation

Implement the client for your provider in `client/my_custom_provider/client.rs`:

```rust
use super::request::prepare_request_body;
use super::stream::process_stream;
use crate::client::LLMClientImpl;
use crate::config::HttpClientOptions;
use crate::errors::llm::LLMError;
use crate::message::common::llm_message::{LLMEvent, LLMRequest};
use crate::message::message::Message;
use crate::message::usage::Usage;
use secrecy::ExposeSecret;
use std::ops::AddAssign;
use tokio::sync::mpsc::UnboundedReceiver;

pub struct MyCustomProviderClient {
    total_usages: Usage,
    history: Vec<Message>,
    client: reqwest::Client,
    config: HttpClientOptions,
}

impl AddAssign<Message> for MyCustomProviderClient {
    fn add_assign(&mut self, rhs: Message) {
        self.history.push(rhs);
    }
}

impl MyCustomProviderClient {
    pub fn new(options: HttpClientOptions) -> anyhow::Result<Self, LLMError> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", options.api_key.expose_secret())
                .parse()
                .unwrap(),
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );

        let reqwest_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|_| LLMError::CreateClientError("Failed to create client".to_string()))?;

        Ok(MyCustomProviderClient {
            client: reqwest_client,
            history: Vec::new(),
            total_usages: Usage::default(),
            config: options,
        })
    }

    pub fn get_usage(&self) -> Usage {
        self.total_usages.clone()
    }

    pub fn get_messages(&self) -> Vec<Message> {
        self.history.clone()
    }
}

impl LLMClientImpl for MyCustomProviderClient {
    async fn count_tokens(&self, request: LLMRequest) -> Result<u32, LLMError> {
        // Implement token counting for your provider
        // If your provider doesn't have a token counting endpoint,
        // you can use an approximation like OpenAI does
        
        let body = prepare_request_body(&request)?;
        let request_json = serde_json::to_string(&body).unwrap();
        let estimated_tokens = request_json.len() / 4;
        
        Ok(estimated_tokens as u32)
    }

    async fn send(&self, request: LLMRequest) -> Result<UnboundedReceiver<LLMEvent>, LLMError> {
        // Validate config
        let base_url = self
            .config
            .api_base
            .clone()
            .ok_or_else(|| LLMError::InvalidConfig("API base URL is not set".to_string()))?;

        // Prepare request body
        let body = prepare_request_body(&request)?;

        // Send request
        let response = self
            .client
            .post(format!("{}/v1/chat/completions", base_url))
            .json(&body)
            .send()
            .await
            .map_err(|_| LLMError::ReqwestError)?;

        // Handle error responses
        if !response.status().is_success() {
            return Err(LLMError::InvalidResponseCode(
                response.status().as_u16(),
                response.text().await.unwrap_or_default(),
            ));
        }

        // Process the stream and return the receiver
        Ok(process_stream(request.model, response).await)
    }

    fn get_config(&self) -> HttpClientOptions {
        self.config.clone()
    }
}
```

### Request Formatting

Implement request formatting for your provider in `client/my_custom_provider/request.rs`:

```rust
use crate::errors::llm::LLMError;
use crate::message::common::llm_message::LLMRequest;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct MyCustomProviderRequest {
    pub model: String,
    pub messages: Vec<MyCustomProviderMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MyCustomProviderMessage {
    pub role: String,
    pub content: String,
}

pub fn prepare_request_body(request: &LLMRequest) -> Result<MyCustomProviderRequest, LLMError> {
    // Convert LLMRequest to your provider's format
    let messages = request
        .messages
        .iter()
        .map(|message| {
            let content = message.text.clone().unwrap_or_default();
            MyCustomProviderMessage {
                role: message.role.clone(),
                content,
            }
        })
        .collect();

    let tools = if let Some(tools) = &request.tools {
        let tools_json: Vec<Value> = tools
            .iter()
            .map(|tool| {
                serde_json::json!({
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.input,
                })
            })
            .collect();
        Some(tools_json)
    } else {
        None
    };

    Ok(MyCustomProviderRequest {
        model: request.model.model_name().to_string(),
        messages,
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        top_p: request.top_p,
        stream: Some(true), // Always use streaming
        tools,
    })
}
```

### Stream Processing

Implement stream processing for your provider in `client/my_custom_provider/stream/processor.rs`:

```rust
use crate::message::common::llm_message::LLMEvent;
use crate::message::content::{Content, ContentDelta, TextContent};
use crate::models::llm_model::LLMModel;
use crate::stop_reason::StopReason;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

#[derive(Debug, Serialize, Deserialize)]
struct MyCustomProviderStreamEvent {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<MyCustomProviderChoice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MyCustomProviderChoice {
    index: usize,
    delta: MyCustomProviderDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MyCustomProviderDelta {
    role: Option<String>,
    content: Option<String>,
}

pub async fn process_stream(
    model: LLMModel,
    response: Response,
) -> UnboundedReceiver<LLMEvent> {
    let (tx, rx) = mpsc::unbounded_channel();
    
    tokio::spawn(async move {
        let stream = response.bytes_stream().eventsource();
        let mut content_index = 0;
        let mut current_content = Content::Text(TextContent::new("".to_string()));
        
        tokio::pin!(stream);
        
        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => {
                    if event.data == "[DONE]" {
                        // End of stream
                        tx.send(LLMEvent::Stop(Some(StopReason::EndTurn))).ok();
                        break;
                    }
                    
                    match serde_json::from_str::<MyCustomProviderStreamEvent>(&event.data) {
                        Ok(stream_event) => {
                            for choice in stream_event.choices {
                                if let Some(content) = choice.delta.content {
                                    if !content.is_empty() {
                                        // Send partial content
                                        current_content.extend_text(&content);
                                        tx.send(LLMEvent::PartialContent(
                                            content_index,
                                            Content::Text(TextContent::new(content)),
                                        )).ok();
                                    }
                                }
                                
                                if let Some(finish_reason) = choice.finish_reason {
                                    // Send stop event
                                    let stop_reason = match finish_reason.as_str() {
                                        "stop" => StopReason::EndTurn,
                                        "length" => StopReason::MaxTokens,
                                        "tool_calls" => StopReason::ToolUse,
                                        _ => StopReason::EndTurn,
                                    };
                                    
                                    tx.send(LLMEvent::Stop(Some(stop_reason))).ok();
                                    break;
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Error parsing stream event: {}", e);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Error in stream: {}", e);
                    tx.send(LLMEvent::Stop(None)).ok();
                    break;
                }
            }
        }
        
        // Send final content
        tx.send(LLMEvent::Content(current_content)).ok();
    });
    
    rx
}
```

## Model Definition

Define models for your provider:

```rust
lazy_static! {
    pub static ref MyCustomModel: LLMModel = LLMModelBuilder::new()
        .model_name("my-custom-model")
        .display_name("My Custom Model")
        .provider(LLMProvider::MyCustomProvider)
        .capabilities(
            ModelCapabilitiesBuilder::new()
                .with_streaming()
                .build()
        )
        .cost_calculation(Arc::new(my_custom_model_cost_calculation))
        .build();
}

fn my_custom_model_cost_calculation(usage: Usage) -> f64 {
    let input_cost = usage.input_tokens as f64 / 1_000_000.0 * 1.00;
    let output_cost = usage.output_tokens as f64 / 1_000_000.0 * 2.00;
    (input_cost + output_cost).max(0.0)
}
```

## Integration

Update the `AllModels` list to include your models:

```rust
pub static ref AllModels: Vec<&'static LLMModel> = vec![
    // Existing models...
    &MyCustomModel,
];
```

## Complete Example

Here's a complete example of using your custom provider:

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::message::common::llm_message::{LLMEvent, LLMMessage, LLMRequest};
use llm_sdk::message::content::Content;
use llm_sdk::models::llm_model::MyCustomModel;
use secrecy::SecretString;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let api_key = SecretString::new("your-api-key".to_string());
    let options = HttpClientOptions::new_my_custom_provider(api_key);
    let client = LLMClient::new_my_custom_provider(options)?;
    
    // Prepare a request
    let request = LLMRequest {
        model: MyCustomModel.clone(),
        messages: vec![
            LLMMessage {
                role: "user".to_string(),
                text: Some("Hello, world!".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    
    // Send the request
    let mut receiver = client.send(request).await?;
    
    // Process the response
    while let Some(event) = receiver.recv().await {
        match event {
            LLMEvent::PartialContent(_, content) => {
                if let Content::Text(text) = &content {
                    print!("{}", text.text);
                    std::io::stdout().flush()?;
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
    
    Ok(())
}
```

This example demonstrates:
1. Creating a client for your custom provider
2. Preparing a request with your custom model
3. Sending the request and processing the response