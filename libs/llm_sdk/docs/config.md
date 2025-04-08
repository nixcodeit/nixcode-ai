# Configuration API Documentation

The Configuration module provides structures and utilities for configuring the LLM SDK. It handles API keys, base URLs, and provider-specific settings.

## Table of Contents

1. [Overview](#overview)
2. [HttpClientOptions](#httpclientoptions)
3. [Provider Configuration](#provider-configuration)
4. [API Key Management](#api-key-management)
5. [Usage Examples](#usage-examples)

## Overview

The configuration module consists of:

- `config.rs`: Defines the `HttpClientOptions` struct for client configuration

This module enables:
- Secure handling of API keys
- Provider-specific configuration
- Custom API base URLs

## HttpClientOptions

The `HttpClientOptions` struct represents the configuration for an HTTP client:

```rust
pub struct HttpClientOptions {
    pub provider: LLMProvider,
    pub api_key: SecretString,
    pub api_base: Option<String>,
}
```

### Fields

- `provider`: The LLM provider (Anthropic, OpenAI, etc.)
- `api_key`: The API key for the provider (securely stored)
- `api_base`: The base URL for the provider's API (optional)

### Methods

- `new_anthropic(api_key: SecretString) -> Self`: Creates configuration for Anthropic
- `new_openai(api_key: SecretString) -> Self`: Creates configuration for OpenAI
- `new_groq(api_key: SecretString) -> Self`: Creates configuration for Groq
- `new_openrouter(api_key: SecretString) -> Self`: Creates configuration for OpenRouter

## Provider Configuration

Each provider has its own configuration method that sets appropriate defaults:

### Anthropic

```rust
pub fn new_anthropic(api_key: SecretString) -> Self {
    Self {
        provider: LLMProvider::Anthropic,
        api_key,
        api_base: Some("https://api.anthropic.com".to_string()),
    }
}
```

### OpenAI

```rust
pub fn new_openai(api_key: SecretString) -> Self {
    Self {
        provider: LLMProvider::OpenAI,
        api_key,
        api_base: Some("https://api.openai.com".to_string()),
    }
}
```

### Groq

```rust
pub fn new_groq(api_key: SecretString) -> Self {
    Self {
        provider: LLMProvider::Groq,
        api_key,
        api_base: Some("https://api.groq.com/openai".to_string()),
    }
}
```

### OpenRouter

```rust
pub fn new_openrouter(api_key: SecretString) -> Self {
    Self {
        provider: LLMProvider::OpenRouter,
        api_key,
        api_base: Some("https://openrouter.ai/api".to_string()),
    }
}
```

## API Key Management

The SDK uses the `secrecy` crate to securely handle API keys:

```rust
use secrecy::SecretString;

let api_key = SecretString::new("your-api-key".to_string());
```

This ensures that API keys are not accidentally logged or exposed in debug output.

To use the API key in a request, you need to explicitly expose it:

```rust
use secrecy::ExposeSecret;

let api_key_str = api_key.expose_secret();
```

## Usage Examples

### Creating Configuration for Anthropic

```rust
use llm_sdk::config::HttpClientOptions;
use secrecy::SecretString;

// Create configuration for Anthropic
let api_key = SecretString::new("your-anthropic-api-key".to_string());
let config = HttpClientOptions::new_anthropic(api_key);
```

### Creating Configuration for OpenAI

```rust
use llm_sdk::config::HttpClientOptions;
use secrecy::SecretString;

// Create configuration for OpenAI
let api_key = SecretString::new("your-openai-api-key".to_string());
let config = HttpClientOptions::new_openai(api_key);
```

### Creating Configuration with Custom API Base

```rust
use llm_sdk::config::HttpClientOptions;
use llm_sdk::providers::LLMProvider;
use secrecy::SecretString;

// Create configuration with custom API base
let api_key = SecretString::new("your-api-key".to_string());
let config = HttpClientOptions {
    provider: LLMProvider::Anthropic,
    api_key,
    api_base: Some("https://your-custom-api-base.com".to_string()),
};
```

### Using Configuration with a Client

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use secrecy::SecretString;

// Create configuration
let api_key = SecretString::new("your-anthropic-api-key".to_string());
let config = HttpClientOptions::new_anthropic(api_key);

// Create client with configuration
let client = LLMClient::new_anthropic(config).unwrap();
```

### Environment Variables

In a real application, you would typically load API keys from environment variables:

```rust
use llm_sdk::config::HttpClientOptions;
use secrecy::SecretString;
use std::env;

// Load API key from environment variable
let api_key = match env::var("ANTHROPIC_API_KEY") {
    Ok(key) => SecretString::new(key),
    Err(_) => {
        eprintln!("ANTHROPIC_API_KEY environment variable not set");
        std::process::exit(1);
    }
};

// Create configuration
let config = HttpClientOptions::new_anthropic(api_key);
```

### Configuration in a Multi-Provider Application

```rust
use llm_sdk::client::LLMClient;
use llm_sdk::config::HttpClientOptions;
use llm_sdk::providers::LLMProvider;
use secrecy::SecretString;
use std::env;

// Function to get client based on provider
fn get_client(provider: LLMProvider) -> Result<LLMClient, Box<dyn std::error::Error>> {
    match provider {
        LLMProvider::Anthropic => {
            let api_key = SecretString::new(env::var("ANTHROPIC_API_KEY")?);
            let config = HttpClientOptions::new_anthropic(api_key);
            Ok(LLMClient::new_anthropic(config)?)
        },
        LLMProvider::OpenAI => {
            let api_key = SecretString::new(env::var("OPENAI_API_KEY")?);
            let config = HttpClientOptions::new_openai(api_key);
            Ok(LLMClient::new_openai(config)?)
        },
        LLMProvider::Groq => {
            let api_key = SecretString::new(env::var("GROQ_API_KEY")?);
            let config = HttpClientOptions::new_groq(api_key);
            Ok(LLMClient::new_openai(config)?)
        },
        LLMProvider::OpenRouter => {
            let api_key = SecretString::new(env::var("OPENROUTER_API_KEY")?);
            let config = HttpClientOptions::new_openrouter(api_key);
            Ok(LLMClient::new_openai(config)?)
        },
        _ => Err("Unsupported provider".into()),
    }
}
```