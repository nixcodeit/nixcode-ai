# Error Handling Documentation

The Error Handling module provides a standardized way to handle errors in the LLM SDK. It defines error types and conversions between different error formats.

## Table of Contents

1. [Overview](#overview)
2. [LLMError Enum](#llmerror-enum)
3. [Error Conversion](#error-conversion)
4. [Provider-Specific Error Handling](#provider-specific-error-handling)
5. [Error Propagation](#error-propagation)
6. [Usage Examples](#usage-examples)

## Overview

The error handling module consists of:

- `errors/llm.rs`: Defines the `LLMError` enum for different error types
- `errors/mod.rs`: Re-exports error types

This module enables:
- Standardized error handling across the SDK
- Conversion between different error formats
- Detailed error information for debugging

## LLMError Enum

The `LLMError` enum represents different types of errors that can occur in the SDK:

```rust
pub enum LLMError {
    CreateClientError(String),
    InvalidRequest,
    InvalidResponseCode(u16, String),
    InvalidResponse(String),
    ParseError(String),
    ReqwestError,
    NetworkError,
    Timeout,
    InputTooLong,
    MissingAPIKey,
    ConversionError(String),
    Generic(String),
    InvalidConfig(String),
}
```

### Error Types

- `CreateClientError`: Error creating an HTTP client
- `InvalidRequest`: Invalid request format
- `InvalidResponseCode`: Unexpected HTTP response code
- `InvalidResponse`: Invalid response format
- `ParseError`: Error parsing JSON or other formats
- `ReqwestError`: Error from the reqwest HTTP client
- `NetworkError`: Network-related error
- `Timeout`: Request timeout
- `InputTooLong`: Input exceeds token limit
- `MissingAPIKey`: API key not provided
- `ConversionError`: Error converting between formats
- `Generic`: Generic error with message
- `InvalidConfig`: Invalid configuration

## Error Conversion

The `LLMError` enum can be converted to other error types:

### Conversion to anyhow::Error

```rust
impl Into<Error> for LLMError {
    fn into(self) -> Error {
        match self {
            LLMError::CreateClientError(e) => Error::msg(e),
            LLMError::InvalidRequest => Error::msg("Invalid request"),
            LLMError::InvalidResponseCode(code, body) => Error::msg(format!(
                "Invalid response code: {} with body: {}",
                code, body
            )),
            // Other conversions...
        }
    }
}
```

### Conversion to Provider-Specific Errors

```rust
impl Into<ErrorEventContent> for LLMError {
    fn into(self) -> ErrorEventContent {
        ErrorEventContent {
            r#type: match self {
                LLMError::CreateClientError(_) => "create_client_error".into(),
                LLMError::InvalidRequest => "invalid_request".into(),
                // Other conversions...
            },
            message: match self {
                LLMError::CreateClientError(e) => e,
                LLMError::InvalidRequest => "Invalid request".into(),
                // Other conversions...
            },
        }
    }
}
```

## Provider-Specific Error Handling

Different providers have different error formats:

### Anthropic Errors

Anthropic errors are handled in the `anthropic/stream/error_handler.rs` file:

```rust
pub fn handle_error(error: Error) -> LLMEvent {
    // Convert error to LLMEvent
}
```

### OpenAI Errors

OpenAI errors are handled in the `openai/stream/error_handler.rs` file:

```rust
pub fn handle_error(error: Error) -> LLMEvent {
    // Convert error to LLMEvent
}
```

## Error Propagation

Errors are propagated through the SDK using the `Result` type:

```rust
pub async fn send(&self, request: LLMRequest) -> Result<UnboundedReceiver<LLMEvent>, LLMError> {
    // Send request and handle errors
}
```

Functions that can fail return a `Result<T, LLMError>` where `T` is the success type and `LLMError` is the error type.

## Usage Examples

### Handling Errors

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::errors::llm::LLMError;
use llm_sdk::message::common::llm_message::LLMRequest;
use llm_sdk::models::llm_model::Sonnet37;
use secrecy::SecretString;

async fn example() {
    // Create a client
    let api_key = SecretString::new("your-api-key".to_string());
    let options = HttpClientOptions::new_anthropic(api_key);
    let client = match LLMClient::new_anthropic(options) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Error creating client: {:?}", e);
            return;
        }
    };
    
    // Prepare a request
    let request = LLMRequest {
        model: Sonnet37.clone(),
        messages: vec![
            // Add your messages here
        ],
        ..Default::default()
    };
    
    // Send the request
    let receiver = match client.send(request).await {
        Ok(receiver) => receiver,
        Err(e) => {
            match e {
                LLMError::MissingAPIKey => {
                    eprintln!("API key not provided");
                },
                LLMError::InvalidResponseCode(code, body) => {
                    eprintln!("Invalid response code: {} with body: {}", code, body);
                },
                LLMError::NetworkError => {
                    eprintln!("Network error");
                },
                _ => {
                    eprintln!("Error sending request: {:?}", e);
                }
            }
            return;
        }
    };
    
    // Process the response
    // ...
}
```

### Custom Error Handling

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::errors::llm::LLMError;
use llm_sdk::message::common::llm_message::LLMRequest;
use llm_sdk::models::llm_model::Sonnet37;
use secrecy::SecretString;

// Custom error handler
fn handle_llm_error(error: LLMError) -> String {
    match error {
        LLMError::CreateClientError(e) => format!("Failed to create client: {}", e),
        LLMError::InvalidRequest => "The request format is invalid".to_string(),
        LLMError::InvalidResponseCode(code, body) => {
            format!("Server returned error code {}: {}", code, body)
        },
        LLMError::InvalidResponse(e) => format!("Invalid response: {}", e),
        LLMError::ParseError(e) => format!("Failed to parse response: {}", e),
        LLMError::ReqwestError => "HTTP client error".to_string(),
        LLMError::NetworkError => "Network error".to_string(),
        LLMError::Timeout => "Request timed out".to_string(),
        LLMError::InputTooLong => "Input exceeds token limit".to_string(),
        LLMError::MissingAPIKey => "API key not provided".to_string(),
        LLMError::ConversionError(e) => format!("Format conversion error: {}", e),
        LLMError::Generic(e) => e,
        LLMError::InvalidConfig(e) => format!("Invalid configuration: {}", e),
    }
}

async fn example() -> Result<(), String> {
    // Create a client
    let api_key = SecretString::new("your-api-key".to_string());
    let options = HttpClientOptions::new_anthropic(api_key);
    let client = LLMClient::new_anthropic(options).map_err(handle_llm_error)?;
    
    // Prepare a request
    let request = LLMRequest {
        model: Sonnet37.clone(),
        messages: vec![
            // Add your messages here
        ],
        ..Default::default()
    };
    
    // Send the request
    let receiver = client.send(request).await.map_err(handle_llm_error)?;
    
    // Process the response
    // ...
    
    Ok(())
}
```

### Error Handling with anyhow

```rust
use anyhow::Result;
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::message::common::llm_message::LLMRequest;
use llm_sdk::models::llm_model::Sonnet37;
use secrecy::SecretString;

async fn example() -> Result<()> {
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
    let receiver = client.send(request).await?;
    
    // Process the response
    // ...
    
    Ok(())
}
```