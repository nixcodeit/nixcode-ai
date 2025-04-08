# Token Counting Documentation

This document explains how token counting works in the LLM SDK. Token counting is important for estimating costs, managing context limits, and optimizing requests.

## Table of Contents

1. [Overview](#overview)
2. [Token Counting Methods](#token-counting-methods)
3. [Provider-Specific Implementations](#provider-specific-implementations)
4. [Usage Tracking](#usage-tracking)
5. [Handling Token Limits](#handling-token-limits)
6. [Usage Examples](#usage-examples)

## Overview

Tokens are the basic units that LLMs process. A token can be as short as one character or as long as one word. Token counting is important for:

- Estimating costs (providers charge per token)
- Managing context limits (models have maximum token limits)
- Optimizing requests (reducing tokens can improve performance)

The LLM SDK provides methods for counting tokens in requests and tracking token usage in responses.

## Token Counting Methods

The `LLMClient` interface provides a `count_tokens` method:

```rust
pub async fn count_tokens(&self, request: LLMRequest) -> Result<u32, LLMError> {
    match self {
        LLMClient::OpenAI(client) => client.count_tokens(request).await,
        LLMClient::Anthropic(client) => client.count_tokens(request).await,
    }
}
```

This method estimates the number of tokens in a request before sending it.

## Provider-Specific Implementations

Different providers have different ways of counting tokens:

### Anthropic

Anthropic provides a dedicated endpoint for token counting:

```rust
async fn count_tokens(&self, request: LLMRequest) -> Result<u32, LLMError> {
    let request = Request::try_from(&request)?;
    let body = serde_json::to_value(&request)
        .map_err(|e| LLMError::ParseError(format!("Failed to serialize request: {}", e)))?;

    let response = self
        .client
        .post("https://api.anthropic.com/v1/messages/count_tokens")
        .json(&body)
        .send()
        .await
        .map_err(|_| LLMError::ReqwestError)?;

    if !response.status().is_success() {
        return Err(LLMError::InvalidResponseCode(
            response.status().as_u16(),
            response.text().await.unwrap_or_default(),
        ));
    }

    let body = response
        .json::<InputTokens>()
        .await
        .map_err(|e| LLMError::InvalidResponse(e.to_string()))?;

    Ok(body.input_tokens)
}
```

### OpenAI

OpenAI doesn't have a dedicated token counting endpoint, so the SDK uses an approximation:

```rust
async fn count_tokens(&self, request: LLMRequest) -> Result<u32, LLMError> {
    // OpenAI doesn't have a dedicated token counting endpoint like Anthropic
    // We use a simple approximation based on JSON size
    let openai_body = request_to_openai(&request);
    let request_json = serde_json::to_string(&openai_body).unwrap();

    // This is a very rough approximation (4 chars ~= 1 token)
    // For production, consider using tiktoken or a similar library
    let estimated_tokens = request_json.len() / 4;

    Ok(estimated_tokens as u32)
}
```

## Usage Tracking

The SDK tracks token usage in responses:

```rust
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_reads: Option<u32>,
    pub cache_writes: Option<u32>,
}
```

Provider-specific usage structures are also defined:

```rust
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

## Handling Token Limits

Models have maximum token limits for context windows. The SDK helps manage these limits:

1. **Count tokens before sending**: Use `count_tokens` to check if a request exceeds the limit
2. **Set max_tokens**: Use the `max_tokens` field in `LLMRequest` to limit response length
3. **Track usage**: Monitor token usage to optimize requests

## Usage Examples

### Counting Tokens

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::message::common::llm_message::{LLMMessage, LLMRequest};
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
                text: Some("Hello, world!".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    
    // Count tokens
    let token_count = client.count_tokens(request.clone()).await?;
    println!("Token count: {}", token_count);
    
    Ok(())
}
```

### Checking Token Limits

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::message::common::llm_message::{LLMMessage, LLMRequest};
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
                text: Some("Hello, world!".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    
    // Count tokens
    let token_count = client.count_tokens(request.clone()).await?;
    
    // Check if the request exceeds the limit
    const MAX_TOKENS: u32 = 200000; // Claude 3.7 Sonnet limit
    if token_count > MAX_TOKENS {
        println!("Request exceeds token limit: {} > {}", token_count, MAX_TOKENS);
        return Ok(());
    }
    
    // Send the request
    let mut receiver = client.send(request).await?;
    
    // Process the response
    // ...
    
    Ok(())
}
```

### Setting Maximum Response Length

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::message::common::llm_message::{LLMMessage, LLMRequest};
use llm_sdk::models::llm_model::Sonnet37;
use secrecy::SecretString;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let api_key = SecretString::new("your-api-key".to_string());
    let options = HttpClientOptions::new_anthropic(api_key);
    let client = LLMClient::new_anthropic(options)?;
    
    // Prepare a request with max_tokens
    let request = LLMRequest {
        model: Sonnet37.clone(),
        messages: vec![
            LLMMessage {
                role: "user".to_string(),
                text: Some("Write a long essay about artificial intelligence.".to_string()),
                ..Default::default()
            },
        ],
        max_tokens: Some(1000), // Limit response to 1000 tokens
        ..Default::default()
    };
    
    // Send the request
    let mut receiver = client.send(request).await?;
    
    // Process the response
    // ...
    
    Ok(())
}
```

### Optimizing Requests

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::message::common::llm_message::{LLMMessage, LLMRequest};
use llm_sdk::models::llm_model::Sonnet37;
use secrecy::SecretString;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let api_key = SecretString::new("your-api-key".to_string());
    let options = HttpClientOptions::new_anthropic(api_key);
    let client = LLMClient::new_anthropic(options)?;
    
    // Function to optimize messages
    fn optimize_messages(messages: Vec<LLMMessage>) -> Vec<LLMMessage> {
        // In a real application, you would implement more sophisticated optimization
        // For example, summarizing long messages, removing redundant information, etc.
        
        // This is a simple example that truncates long messages
        messages.into_iter().map(|mut message| {
            if let Some(text) = &message.text {
                if text.len() > 1000 {
                    message.text = Some(format!("{}...", &text[0..1000]));
                }
            }
            message
        }).collect()
    }
    
    // Prepare a request with optimized messages
    let original_messages = vec![
        LLMMessage {
            role: "user".to_string(),
            text: Some("Hello, world! ".repeat(1000)), // Very long message
            ..Default::default()
        },
    ];
    
    // Optimize messages
    let optimized_messages = optimize_messages(original_messages);
    
    // Create request with optimized messages
    let request = LLMRequest {
        model: Sonnet37.clone(),
        messages: optimized_messages,
        ..Default::default()
    };
    
    // Count tokens
    let token_count = client.count_tokens(request.clone()).await?;
    println!("Token count after optimization: {}", token_count);
    
    // Send the request
    let mut receiver = client.send(request).await?;
    
    // Process the response
    // ...
    
    Ok(())
}
```