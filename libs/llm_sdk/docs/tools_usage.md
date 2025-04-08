# Tool/Function Calling Documentation

This document explains how to use the tool/function calling capabilities of the LLM SDK. Tool calling allows LLMs to request the execution of external functions and receive the results.

## Table of Contents

1. [Overview](#overview)
2. [Tool Definition](#tool-definition)
3. [Including Tools in Requests](#including-tools-in-requests)
4. [Handling Tool Calls](#handling-tool-calls)
5. [Returning Tool Results](#returning-tool-results)
6. [Continuing the Conversation](#continuing-the-conversation)
7. [Provider-Specific Considerations](#provider-specific-considerations)
8. [Complete Example](#complete-example)

## Overview

Tool calling (also known as function calling) allows LLMs to:

1. Request the execution of external functions
2. Receive the results of those functions
3. Continue the conversation with the context of the function results

This enables LLMs to perform actions they couldn't do on their own, such as:
- Retrieving real-time information
- Performing calculations
- Interacting with external systems
- Accessing databases
- Manipulating files

## Tool Definition

Tools are defined using the `Tool` struct:

```rust
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input: Value,
}
```

The `input` field is a JSON schema that defines the parameters for the tool.

### Example Tool Definitions

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

// Define a tool for searching
let search_tool = Tool::new(
    "search".to_string(),
    "Search for information on the web".to_string(),
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
```

### Using JSON Schema from Rust Types

You can generate JSON schema from Rust types using the `json_schema` module:

```rust
use llm_sdk::json_schema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct WeatherParams {
    location: String,
}

let schema = json_schema::schema_from_type::<WeatherParams>();
let weather_tool = Tool::new(
    "get_weather".to_string(),
    "Get the current weather for a location".to_string(),
    schema,
);
```

## Including Tools in Requests

To make tools available to the LLM, include them in the request:

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
    // ...
    
    Ok(())
}
```

## Handling Tool Calls

When the LLM decides to call a tool, it will send a `ToolUseContent` event:

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::message::common::llm_message::{LLMEvent, LLMMessage, LLMRequest};
use llm_sdk::message::content::{Content, ToolUseContent};
use llm_sdk::models::llm_model::Sonnet37;
use llm_sdk::tools::Tool;
use secrecy::SecretString;
use serde_json::json;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // ... (client creation and request setup)
    
    // Send the request
    let mut receiver = client.send(request).await?;
    
    // Process the response
    while let Some(event) = receiver.recv().await {
        match event {
            LLMEvent::Content(Content::ToolUse(tool_use)) => {
                println!("Tool call: {}", tool_use.name);
                println!("Tool input: {:?}", tool_use.input);
                
                // Handle the tool call based on the name
                match tool_use.name.as_str() {
                    "get_weather" => {
                        // Extract parameters
                        let location = tool_use.input.get("location")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        
                        println!("Getting weather for: {}", location);
                        
                        // Execute the tool (in a real app, you would call a weather API)
                        // ...
                    },
                    _ => {
                        println!("Unknown tool: {}", tool_use.name);
                    }
                }
            },
            // Handle other events
            _ => {
                println!("Other event: {:?}", event);
            }
        }
    }
    
    Ok(())
}
```

## Returning Tool Results

After executing a tool, you need to return the results to the LLM:

```rust
use llm_sdk::message::content::ToolResultContent;
use serde_json::json;

// ... (inside the tool call handler)

// Create a tool result
let tool_result = ToolResultContent::new(
    tool_use.id.clone(),
    json!({
        "temperature": 72,
        "condition": "sunny",
        "humidity": 45
    }),
);

// Now you need to send this result back to the LLM
```

## Continuing the Conversation

To continue the conversation with the tool results:

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::message::common::llm_message::{LLMMessage, LLMRequest};
use llm_sdk::message::content::{ToolResultContent, ToolUseContent};
use llm_sdk::models::llm_model::Sonnet37;
use serde_json::json;

async fn continue_conversation(
    client: &LLMClient,
    previous_messages: Vec<LLMMessage>,
    tool_use: ToolUseContent,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a tool result
    let tool_result = ToolResultContent::new(
        tool_use.id.clone(),
        json!({
            "temperature": 72,
            "condition": "sunny",
            "humidity": 45
        }),
    );
    
    // Create a new request with the tool result
    let mut messages = previous_messages;
    
    // Add the assistant's tool call
    messages.push(LLMMessage {
        role: "assistant".to_string(),
        tool_calls: Some(vec![tool_use]),
        ..Default::default()
    });
    
    // Add the tool result
    messages.push(LLMMessage {
        role: "user".to_string(),
        tool_results: Some(vec![tool_result]),
        ..Default::default()
    });
    
    // Create a new request
    let request = LLMRequest {
        model: Sonnet37.clone(),
        messages,
        ..Default::default()
    };
    
    // Send the request
    let mut receiver = client.send(request).await?;
    
    // Process the response
    // ...
    
    Ok(())
}
```

## Provider-Specific Considerations

Different providers have different ways of handling tools:

### Anthropic

Anthropic uses the `tools` field in the request and expects tool results to be returned as user messages.

### OpenAI

OpenAI uses the `functions` field in the request and expects tool results to be returned as function call results.

The SDK handles these differences internally, providing a unified interface for tool definitions and results.

## Complete Example

Here's a complete example of using tools with the LLM SDK:

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::message::common::llm_message::{LLMEvent, LLMMessage, LLMRequest};
use llm_sdk::message::content::{Content, ToolResultContent, ToolUseContent};
use llm_sdk::models::llm_model::Sonnet37;
use llm_sdk::tools::Tool;
use secrecy::SecretString;
use serde_json::json;
use std::io::Write;

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
    
    let calculator_tool = Tool::new(
        "calculate".to_string(),
        "Perform a mathematical calculation".to_string(),
        json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "The mathematical expression to evaluate"
                }
            },
            "required": ["expression"]
        }),
    );
    
    // Create initial messages
    let messages = vec![
        LLMMessage {
            role: "user".to_string(),
            text: Some("What's the weather like in New York? Also, what's 123 * 456?".to_string()),
            ..Default::default()
        },
    ];
    
    // Create a request with tools
    let request = LLMRequest {
        model: Sonnet37.clone(),
        messages: messages.clone(),
        tools: Some(vec![weather_tool, calculator_tool]),
        ..Default::default()
    };
    
    // Send the request
    let mut receiver = client.send(request).await?;
    
    // Process the response
    let mut all_messages = messages;
    let mut tool_calls = Vec::new();
    
    while let Some(event) = receiver.recv().await {
        match event {
            LLMEvent::PartialContent(_, content) => {
                if let Content::Text(text) = &content {
                    print!("{}", text.text);
                    std::io::stdout().flush()?;
                }
            },
            LLMEvent::Content(Content::ToolUse(tool_use)) => {
                println!("\nTool call: {}", tool_use.name);
                println!("Tool input: {:?}", tool_use.input);
                
                // Store the tool call
                tool_calls.push(tool_use.clone());
                
                // Handle the tool call based on the name
                match tool_use.name.as_str() {
                    "get_weather" => {
                        // Extract parameters
                        let location = tool_use.input.get("location")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        
                        println!("Getting weather for: {}", location);
                        
                        // In a real app, you would call a weather API
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
                        
                        // Add the assistant's tool call to the messages
                        all_messages.push(LLMMessage {
                            role: "assistant".to_string(),
                            tool_calls: Some(vec![tool_use]),
                            ..Default::default()
                        });
                        
                        // Add the tool result to the messages
                        all_messages.push(LLMMessage {
                            role: "user".to_string(),
                            tool_results: Some(vec![tool_result]),
                            ..Default::default()
                        });
                    },
                    "calculate" => {
                        // Extract parameters
                        let expression = tool_use.input.get("expression")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        
                        println!("Calculating: {}", expression);
                        
                        // In a real app, you would use a proper expression evaluator
                        let result = if expression == "123 * 456" {
                            123 * 456
                        } else {
                            0
                        };
                        
                        // Create a tool result
                        let tool_result = ToolResultContent::new(
                            tool_use.id.clone(),
                            json!({
                                "result": result
                            }),
                        );
                        
                        // Add the assistant's tool call to the messages
                        all_messages.push(LLMMessage {
                            role: "assistant".to_string(),
                            tool_calls: Some(vec![tool_use]),
                            ..Default::default()
                        });
                        
                        // Add the tool result to the messages
                        all_messages.push(LLMMessage {
                            role: "user".to_string(),
                            tool_results: Some(vec![tool_result]),
                            ..Default::default()
                        });
                    },
                    _ => {
                        println!("Unknown tool: {}", tool_use.name);
                    }
                }
            },
            LLMEvent::Stop(_) => {
                println!("\nStream ended");
                
                // If there were tool calls, continue the conversation
                if !tool_calls.is_empty() {
                    println!("\nContinuing conversation with tool results...");
                    
                    // Create a new request with the updated messages
                    let continue_request = LLMRequest {
                        model: Sonnet37.clone(),
                        messages: all_messages.clone(),
                        tools: Some(vec![]), // No need for tools in the continuation
                        ..Default::default()
                    };
                    
                    // Send the continuation request
                    let mut continue_receiver = client.send(continue_request).await?;
                    
                    // Process the continuation response
                    while let Some(continue_event) = continue_receiver.recv().await {
                        match continue_event {
                            LLMEvent::PartialContent(_, content) => {
                                if let Content::Text(text) = &content {
                                    print!("{}", text.text);
                                    std::io::stdout().flush()?;
                                }
                            },
                            LLMEvent::Stop(_) => {
                                println!("\nContinuation ended");
                                break;
                            },
                            _ => {
                                println!("\nOther event: {:?}", continue_event);
                            }
                        }
                    }
                }
                
                break;
            },
            _ => {
                println!("\nOther event: {:?}", event);
            }
        }
    }
    
    Ok(())
}
```

This example demonstrates:
1. Defining multiple tools
2. Handling tool calls
3. Executing tools (simulated)
4. Returning tool results
5. Continuing the conversation with tool results