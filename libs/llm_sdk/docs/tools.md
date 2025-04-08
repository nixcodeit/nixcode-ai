# Tools API Documentation

The Tools module provides support for function calling with LLMs. It defines the structure for tool definitions and handles JSON schema validation for tool parameters.

## Table of Contents

1. [Overview](#overview)
2. [Tool Structure](#tool-structure)
3. [JSON Schema](#json-schema)
4. [Tool Usage in Messages](#tool-usage-in-messages)
5. [Tool Results](#tool-results)
6. [Provider-Specific Implementations](#provider-specific-implementations)
7. [Usage Examples](#usage-examples)

## Overview

The tools module consists of:

- `tools.rs`: Defines the `Tool` struct for function definitions
- `json_schema.rs`: Handles JSON schema for tool parameters

This module enables LLMs to:
1. Request the execution of external functions
2. Receive the results of those functions
3. Continue the conversation with the context of the function results

## Tool Structure

The `Tool` struct represents a function that an LLM can call:

```rust
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input: Value,
}
```

### Fields

- `name`: The name of the tool/function
- `description`: A description of what the tool does
- `input`: A JSON schema defining the parameters for the tool

### Methods

- `new(name: String, description: String, input: Value) -> Self`: Creates a new tool
- `with_name(name: String) -> Self`: Sets the name of the tool
- `with_description(description: String) -> Self`: Sets the description of the tool
- `with_input(input: Value) -> Self`: Sets the input schema for the tool

## JSON Schema

The `json_schema.rs` module provides utilities for working with JSON schemas:

```rust
pub fn schema_from_type<T>() -> Value
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    // Generate a JSON schema from a Rust type
}
```

This function generates a JSON schema from a Rust type, which can be used as the `input` field for a `Tool`.

## Tool Usage in Messages

Tools are used in messages through the `ToolUseContent` struct:

```rust
pub struct ToolUseContent {
    pub id: String,
    pub name: String,
    pub input: Value,
}
```

### Fields

- `id`: A unique identifier for the tool call
- `name`: The name of the tool being called
- `input`: The parameters for the tool call as a JSON value

### Methods

- `new(id: String, name: String, input: Value) -> Self`: Creates a new tool use content
- `validate_content() -> bool`: Validates the tool use content

## Tool Results

Tool results are represented by the `ToolResultContent` struct:

```rust
pub struct ToolResultContent {
    pub tool_call_id: String,
    pub result: Value,
}
```

### Fields

- `tool_call_id`: The ID of the tool call this result is for
- `result`: The result of the tool call as a JSON value

### Methods

- `new(tool_call_id: String, result: Value) -> Self`: Creates a new tool result content
- `validate_content() -> bool`: Validates the tool result content

## Provider-Specific Implementations

Different LLM providers have different ways of handling tools:

### Anthropic

Anthropic uses the `tools` field in the request to define available tools:

```rust
{
    "model": "claude-3-7-sonnet-latest",
    "messages": [...],
    "tools": [
        {
            "name": "get_weather",
            "description": "Get the current weather for a location",
            "input_schema": {
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The location to get weather for"
                    }
                },
                "required": ["location"]
            }
        }
    ]
}
```

### OpenAI

OpenAI uses the `functions` field in the request to define available tools:

```rust
{
    "model": "gpt-4o",
    "messages": [...],
    "functions": [
        {
            "name": "get_weather",
            "description": "Get the current weather for a location",
            "parameters": {
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The location to get weather for"
                    }
                },
                "required": ["location"]
            }
        }
    ]
}
```

The SDK handles these differences internally, providing a unified interface for tool definitions.

## Usage Examples

### Defining Tools

```rust
use llm_sdk::tools::Tool;
use serde_json::json;

// Define a tool for getting weather
let weather_tool = Tool::new(
    "get_weather".to_string(),
    "Get the current weather for a location".to_string(),
    json!({
        "type": "object",
        "properties": {
            "location": {
                "type": "string",
                "description": "The location to get weather for"
            }
        },
        "required": ["location"]
    }),
);

// Define a tool for searching the web
let search_tool = Tool::new(
    "search_web".to_string(),
    "Search the web for information".to_string(),
    json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "The search query"
            }
        },
        "required": ["query"]
    }),
);

// Create a list of tools
let tools = vec![weather_tool, search_tool];
```

### Using Tools in Requests

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::message::common::llm_message::{LLMMessage, LLMRequest};
use llm_sdk::models::llm_model::Sonnet37;
use llm_sdk::tools::Tool;
use secrecy::SecretString;
use serde_json::json;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let api_key = SecretString::new("your-api-key".to_string());
    let options = HttpClientOptions::new_anthropic(api_key);
    let client = LLMClient::new_anthropic(options)?;
    
    // Define tools
    let weather_tool = Tool::new(
        "get_weather".to_string(),
        "Get the current weather for a location".to_string(),
        json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "The location to get weather for"
                }
            },
            "required": ["location"]
        }),
    );
    
    // Create a request with tools
    let request = LLMRequest {
        model: Sonnet37.clone(),
        messages: vec![
            LLMMessage {
                role: "user".to_string(),
                text: Some("What's the weather like in New York?".to_string()),
                ..Default::default()
            },
        ],
        tools: Some(vec![weather_tool]),
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

### Handling Tool Calls

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::message::common::llm_message::{LLMEvent, LLMMessage, LLMRequest};
use llm_sdk::message::content::{Content, ToolResultContent, ToolUseContent};
use llm_sdk::models::llm_model::Sonnet37;
use llm_sdk::tools::Tool;
use secrecy::SecretString;
use serde_json::json;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let api_key = SecretString::new("your-api-key".to_string());
    let options = HttpClientOptions::new_anthropic(api_key);
    let client = LLMClient::new_anthropic(options)?;
    
    // Define tools
    let weather_tool = Tool::new(
        "get_weather".to_string(),
        "Get the current weather for a location".to_string(),
        json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "The location to get weather for"
                }
            },
            "required": ["location"]
        }),
    );
    
    // Create a request with tools
    let request = LLMRequest {
        model: Sonnet37.clone(),
        messages: vec![
            LLMMessage {
                role: "user".to_string(),
                text: Some("What's the weather like in New York?".to_string()),
                ..Default::default()
            },
        ],
        tools: Some(vec![weather_tool]),
        ..Default::default()
    };
    
    // Send the request
    let mut receiver = client.send(request).await?;
    
    // Process the response
    let mut tool_calls = Vec::new();
    
    while let Some(event) = receiver.recv().await {
        match event {
            LLMEvent::Content(Content::ToolUse(tool_use)) => {
                // Store the tool call
                tool_calls.push(tool_use);
                
                // Execute the tool
                if tool_use.name == "get_weather" {
                    let location = tool_use.input.get("location").unwrap().as_str().unwrap();
                    
                    // In a real application, you would call a weather API here
                    let weather_result = json!({
                        "temperature": 72,
                        "condition": "sunny",
                        "humidity": 45
                    });
                    
                    // Create a tool result
                    let tool_result = ToolResultContent::new(
                        tool_use.id.clone(),
                        weather_result,
                    );
                    
                    // Send the tool result back to the LLM
                    let continue_request = LLMRequest {
                        model: Sonnet37.clone(),
                        messages: vec![
                            // Include previous messages...
                            LLMMessage {
                                role: "assistant".to_string(),
                                tool_calls: Some(vec![tool_use]),
                                ..Default::default()
                            },
                            LLMMessage {
                                role: "user".to_string(),
                                tool_results: Some(vec![tool_result]),
                                ..Default::default()
                            },
                        ],
                        ..Default::default()
                    };
                    
                    // Send the continuation request
                    let mut continue_receiver = client.send(continue_request).await?;
                    
                    // Process the continuation response
                    while let Some(continue_event) = continue_receiver.recv().await {
                        // Handle the continuation event
                        println!("{:?}", continue_event);
                    }
                }
            },
            _ => {
                // Handle other events
                println!("{:?}", event);
            }
        }
    }
    
    Ok(())
}
```