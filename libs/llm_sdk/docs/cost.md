# Cost Calculation Documentation

This document explains how cost calculation works in the LLM SDK. Cost calculation is important for tracking and managing expenses when using LLM providers.

## Table of Contents

1. [Overview](#overview)
2. [Cost Calculation Functions](#cost-calculation-functions)
3. [Model-Specific Cost Calculations](#model-specific-cost-calculations)
4. [Usage Tracking](#usage-tracking)
5. [Calculating Total Costs](#calculating-total-costs)
6. [Usage Examples](#usage-examples)

## Overview

Different LLM providers charge different rates for their models, typically based on:

- Input tokens (tokens in the request)
- Output tokens (tokens in the response)
- Cache operations (for models that support caching)

The LLM SDK provides a flexible system for calculating costs based on token usage.

## Cost Calculation Functions

Cost calculation functions are defined as:

```rust
pub type CostCalculation = Arc<dyn Fn(Usage) -> f64 + Send + Sync>;
```

These functions take a `Usage` struct and return a cost in USD.

The `Usage` struct contains:

```rust
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_reads: Option<u32>,
    pub cache_writes: Option<u32>,
}
```

## Model-Specific Cost Calculations

Each model has its own cost calculation function:

### Claude 3.7 Sonnet

```rust
fn sonnet37_cost_calculation(usage: Usage) -> f64 {
    let create_input_cache_cost = usage.cache_writes.unwrap_or(0) as f64 / 1_000_000.0 * 3.00;
    let read_input_cache_cost = usage.cache_reads.unwrap_or(0) as f64 / 1_000_000.0 * 3.00;
    let input_cost = usage.input_tokens as f64 / 1_000_000.0 * 6.00;
    let output_cost = usage.output_tokens as f64 / 1_000_000.0 * 24.00;
    (create_input_cache_cost + read_input_cache_cost + input_cost + output_cost).max(0.0)
}
```

### Claude 3.5 Haiku

```rust
fn haiku35_cost_calculation(usage: Usage) -> f64 {
    let create_input_cache_cost = usage.cache_writes.unwrap_or(0) as f64 / 1_000_000.0 * 0.25;
    let read_input_cache_cost = usage.cache_reads.unwrap_or(0) as f64 / 1_000_000.0 * 0.25;
    let input_cost = usage.input_tokens as f64 / 1_000_000.0 * 0.50;
    let output_cost = usage.output_tokens as f64 / 1_000_000.0 * 1.50;
    (create_input_cache_cost + read_input_cache_cost + input_cost + output_cost).max(0.0)
}
```

### GPT-4o

```rust
fn o4_cost_calculation(usage: Usage) -> f64 {
    let create_input_cache_cost = usage.cache_writes.unwrap_or(0) as f64 / 1_000_000.0 * 1.25;
    let read_input_cache_cost = usage.cache_reads.unwrap_or(0) as f64 / 1_000_000.0 * 1.25;
    let input_cost = usage.input_tokens as f64 / 1_000_000.0 * 2.50;
    let output_cost = usage.output_tokens as f64 / 1_000_000.0 * 10.00;
    (create_input_cache_cost + read_input_cache_cost + input_cost + output_cost).max(0.0)
}
```

### GPT-3o Mini

```rust
fn o3mini_cost_calculation(usage: Usage) -> f64 {
    let create_input_cache_cost = usage.cache_writes.unwrap_or(0) as f64 / 1_000_000.0 * 0.55;
    let read_input_cache_cost = usage.cache_reads.unwrap_or(0) as f64 / 1_000_000.0 * 0.55;
    let input_cost = usage.input_tokens as f64 / 1_000_000.0 * 1.10;
    let output_cost = usage.output_tokens as f64 / 1_000_000.0 * 4.40;
    (create_input_cache_cost + read_input_cache_cost + input_cost + output_cost).max(0.0)
}
```

### Llama 4 Scout

```rust
fn llama4_scout_cost_calculation(usage: Usage) -> f64 {
    let input_cost = usage.input_tokens as f64 / 1_000_000.0 * 0.11;
    let output_cost = usage.output_tokens as f64 / 1_000_000.0 * 0.32;
    (input_cost + output_cost).max(0.0)
}
```

### Qwen Qwq 32b

```rust
fn qwen_qwq_32b_cost_calculation(usage: Usage) -> f64 {
    let input_cost = usage.input_tokens as f64 / 1_000_000.0 * 0.29;
    let output_cost = usage.output_tokens as f64 / 1_000_000.0 * 0.39;
    (input_cost + output_cost).max(0.0)
}
```

### DeepSeek R1

```rust
fn deepseek_r1_cost_calculation(usage: Usage) -> f64 {
    let input_cost = usage.input_tokens as f64 / 1_000_000.0 * 0.75;
    let output_cost = usage.output_tokens as f64 / 1_000_000.0 * 0.99;
    (input_cost + output_cost).max(0.0)
}
```

### DeepSeek V3

```rust
fn deepseek_v3_cost_calulcation(usage: Usage) -> f64 {
    let input_cost = usage.input_tokens as f64 / 1_000_000.0 * 0.4;
    let output_cost = usage.output_tokens as f64 / 1_000_000.0 * 0.89;
    (input_cost + output_cost).max(0.0)
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

## Calculating Total Costs

To calculate the cost of a request:

1. Track token usage from the response
2. Use the model's cost calculation function

```rust
let cost = model.calculate_cost(usage);
```

## Usage Examples

### Calculating Cost for a Request

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::message::common::llm_message::{LLMEvent, LLMMessage, LLMRequest};
use llm_sdk::message::usage::Usage;
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
    
    // Count input tokens
    let input_tokens = client.count_tokens(request.clone()).await?;
    
    // Send the request
    let mut receiver = client.send(request).await?;
    
    // Process the response and count output tokens
    let mut output_tokens = 0;
    
    while let Some(event) = receiver.recv().await {
        // In a real application, you would extract token counts from the response
        // For this example, we'll use a placeholder
        output_tokens += 10; // Placeholder
        
        // Check for stop event
        if let LLMEvent::Stop(_) = event {
            break;
        }
    }
    
    // Create usage object
    let usage = Usage {
        input_tokens,
        output_tokens,
        cache_reads: None,
        cache_writes: None,
    };
    
    // Calculate cost
    let cost = Sonnet37.calculate_cost(usage);
    
    println!("Input tokens: {}", input_tokens);
    println!("Output tokens: {}", output_tokens);
    println!("Total cost: ${:.6}", cost);
    
    Ok(())
}
```

### Tracking Costs Across Multiple Requests

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::message::common::llm_message::{LLMEvent, LLMMessage, LLMRequest};
use llm_sdk::message::usage::Usage;
use llm_sdk::models::llm_model::Sonnet37;
use secrecy::SecretString;
use std::sync::Arc;
use tokio::sync::Mutex;

struct CostTracker {
    total_input_tokens: u32,
    total_output_tokens: u32,
    total_cost: f64,
}

impl CostTracker {
    fn new() -> Self {
        Self {
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_cost: 0.0,
        }
    }
    
    fn add_usage(&mut self, usage: Usage, model: &LLMModel) {
        self.total_input_tokens += usage.input_tokens;
        self.total_output_tokens += usage.output_tokens;
        self.total_cost += model.calculate_cost(usage);
    }
    
    fn report(&self) {
        println!("Total input tokens: {}", self.total_input_tokens);
        println!("Total output tokens: {}", self.total_output_tokens);
        println!("Total cost: ${:.6}", self.total_cost);
    }
}

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let api_key = SecretString::new("your-api-key".to_string());
    let options = HttpClientOptions::new_anthropic(api_key);
    let client = LLMClient::new_anthropic(options)?;
    
    // Create a cost tracker
    let cost_tracker = Arc::new(Mutex::new(CostTracker::new()));
    
    // Function to send a request and track cost
    async fn send_request(
        client: &LLMClient,
        message: &str,
        cost_tracker: Arc<Mutex<CostTracker>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Prepare a request
        let request = LLMRequest {
            model: Sonnet37.clone(),
            messages: vec![
                LLMMessage {
                    role: "user".to_string(),
                    text: Some(message.to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        
        // Count input tokens
        let input_tokens = client.count_tokens(request.clone()).await?;
        
        // Send the request
        let mut receiver = client.send(request).await?;
        
        // Process the response and count output tokens
        let mut output_tokens = 0;
        let mut response = String::new();
        
        while let Some(event) = receiver.recv().await {
            match event {
                LLMEvent::PartialContent(_, content) => {
                    if let Content::Text(text) = &content {
                        response.push_str(&text.text);
                        // In a real application, you would extract token counts from the response
                        output_tokens += text.text.len() / 4; // Rough approximation
                    }
                },
                LLMEvent::Stop(_) => {
                    break;
                },
                _ => {},
            }
        }
        
        // Create usage object
        let usage = Usage {
            input_tokens,
            output_tokens,
            cache_reads: None,
            cache_writes: None,
        };
        
        // Update cost tracker
        let mut tracker = cost_tracker.lock().await;
        tracker.add_usage(usage, &Sonnet37);
        
        Ok(response)
    }
    
    // Send multiple requests
    let messages = vec![
        "Tell me about artificial intelligence.",
        "What are the ethical implications of AI?",
        "How does machine learning work?",
    ];
    
    for message in messages {
        let response = send_request(&client, message, cost_tracker.clone()).await?;
        println!("Response: {}", response);
    }
    
    // Report total cost
    let tracker = cost_tracker.lock().await;
    tracker.report();
    
    Ok(())
}
```

### Cost Comparison Between Models

```rust
use llm_sdk::message::usage::Usage;
use llm_sdk::models::llm_model::{Sonnet37, Haiku35, Gpt4o, Gpt3oMini, Llama4, DeepSeekR1};

fn compare_costs() {
    // Create a sample usage
    let usage = Usage {
        input_tokens: 1000,
        output_tokens: 500,
        cache_reads: None,
        cache_writes: None,
    };
    
    // Calculate costs for different models
    let models = [
        ("Claude 3.7 Sonnet", Sonnet37.calculate_cost(usage.clone())),
        ("Claude 3.5 Haiku", Haiku35.calculate_cost(usage.clone())),
        ("GPT-4o", Gpt4o.calculate_cost(usage.clone())),
        ("GPT-3o Mini", Gpt3oMini.calculate_cost(usage.clone())),
        ("Llama 4 Scout", Llama4.calculate_cost(usage.clone())),
        ("DeepSeek R1", DeepSeekR1.calculate_cost(usage.clone())),
    ];
    
    // Print costs
    println!("Cost comparison for {} input tokens and {} output tokens:", usage.input_tokens, usage.output_tokens);
    for (name, cost) in models {
        println!("{}: ${:.6}", name, cost);
    }
}
```