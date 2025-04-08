# Models API Documentation

The Models module defines the LLM models supported by the SDK and their capabilities. It provides a flexible system for defining models, their properties, and cost calculation functions.

## Table of Contents

1. [Overview](#overview)
2. [LLMModel Struct](#llmmodel-struct)
3. [LLMModelBuilder](#llmmodelbuilder)
4. [Model Capabilities](#model-capabilities)
5. [Cost Calculation](#cost-calculation)
6. [Supported Models](#supported-models)
7. [Adding New Models](#adding-new-models)

## Overview

The models module is organized as follows:

```
models/
├── anthropic/
│   ├── haiku35.rs
│   ├── mod.rs
│   └── sonnet37.rs
├── capabilities.rs
├── llm_model.rs
└── mod.rs
```

The module provides:
- A struct for representing LLM models
- A builder pattern for creating models
- A system for defining model capabilities
- Cost calculation functions for different models

## LLMModel Struct

The `LLMModel` struct represents an LLM model with its properties:

```rust
pub struct LLMModel {
    display_name: String,
    model_name: String,
    provider: LLMProvider,
    cost_calculation: CostCalculation,
    capabilities: ModelCapabilities,
}
```

### Fields

- `display_name`: Human-readable name for the model
- `model_name`: API identifier for the model
- `provider`: The LLM provider for this model
- `cost_calculation`: Function for calculating usage cost
- `capabilities`: Model capabilities (streaming, cache, thinking)

### Methods

- `model_name() -> &str`: Returns the API identifier
- `provider() -> &LLMProvider`: Returns the provider
- `full_model_name() -> String`: Returns the full model name (provider/model)
- `calculate_cost(usage: Usage) -> f64`: Calculates usage cost
- `capabilities() -> &ModelCapabilities`: Returns model capabilities

## LLMModelBuilder

The `LLMModelBuilder` provides a builder pattern for creating `LLMModel` instances:

```rust
pub struct LLMModelBuilder {
    display_name: Option<String>,
    model_name: Option<String>,
    provider: LLMProvider,
    cost_calculation: Option<CostCalculation>,
    capabilities: Option<ModelCapabilities>,
}
```

### Methods

- `new() -> Self`: Creates a new builder
- `display_name(display_name: impl Into<String>) -> Self`: Sets the display name
- `model_name(model_name: impl Into<String>) -> Self`: Sets the model name
- `provider(provider: LLMProvider) -> Self`: Sets the provider
- `cost_calculation(cost_calculation: CostCalculation) -> Self`: Sets the cost calculation function
- `capabilities(capabilities: ModelCapabilities) -> Self`: Sets the capabilities
- `build() -> LLMModel`: Builds the model

### Example

```rust
let model = LLMModelBuilder::new()
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

## Model Capabilities

The `ModelCapabilities` struct defines what features a model supports:

```rust
pub struct ModelCapabilities {
    pub supports_streaming: bool,
    pub supports_cache: bool,
    pub supports_thinking: bool,
}
```

### Fields

- `supports_streaming`: Whether the model supports streaming responses
- `supports_cache`: Whether the model supports caching
- `supports_thinking`: Whether the model supports thinking/reasoning

### Methods

- `supports_streaming() -> bool`: Checks if streaming is supported
- `supports_cache() -> bool`: Checks if caching is supported
- `supports_thinking() -> bool`: Checks if thinking is supported

### ModelCapabilitiesBuilder

The `ModelCapabilitiesBuilder` provides a builder pattern for creating `ModelCapabilities`:

```rust
pub struct ModelCapabilitiesBuilder {
    supports_streaming: bool,
    supports_cache: bool,
    supports_thinking: bool,
}
```

#### Methods

- `new() -> Self`: Creates a new builder
- `with_streaming() -> Self`: Enables streaming support
- `with_cache() -> Self`: Enables cache support
- `with_thinking() -> Self`: Enables thinking support
- `build() -> ModelCapabilities`: Builds the capabilities

## Cost Calculation

The SDK includes cost calculation functions for different models:

```rust
pub type CostCalculation = Arc<dyn Fn(Usage) -> f64 + Send + Sync>;
```

### Example Cost Calculation Functions

```rust
fn sonnet37_cost_calculation(usage: Usage) -> f64 {
    let create_input_cache_cost = usage.cache_writes.unwrap_or(0) as f64 / 1_000_000.0 * 3.00;
    let read_input_cache_cost = usage.cache_reads.unwrap_or(0) as f64 / 1_000_000.0 * 3.00;
    let input_cost = usage.input_tokens as f64 / 1_000_000.0 * 6.00;
    let output_cost = usage.output_tokens as f64 / 1_000_000.0 * 24.00;
    (create_input_cache_cost + read_input_cache_cost + input_cost + output_cost).max(0.0)
}

fn haiku35_cost_calculation(usage: Usage) -> f64 {
    let create_input_cache_cost = usage.cache_writes.unwrap_or(0) as f64 / 1_000_000.0 * 0.25;
    let read_input_cache_cost = usage.cache_reads.unwrap_or(0) as f64 / 1_000_000.0 * 0.25;
    let input_cost = usage.input_tokens as f64 / 1_000_000.0 * 0.50;
    let output_cost = usage.output_tokens as f64 / 1_000_000.0 * 1.50;
    (create_input_cache_cost + read_input_cache_cost + input_cost + output_cost).max(0.0)
}
```

## Supported Models

The SDK includes predefined models for different providers:

### Anthropic Models

- `Sonnet37`: Claude 3.7 Sonnet
- `Haiku35`: Claude 3.5 Haiku

### OpenAI Models

- `Gpt4o`: GPT-4o
- `Gpt3oMini`: GPT-3o Mini

### Groq Models

- `Llama4`: Llama 4 Scout
- `QwenQwq32b`: Qwen Qwq 32b
- `DeepSeekR1`: DeepSeek R1

### OpenRouter Models

- `QuasarAlpha`: Quasar Alpha
- `Llama4OpenRouter`: Llama 4 Scout (via OpenRouter)
- `DeepSeekV3`: DeepSeek V3
- `Gemini25Pro`: Gemini 2.5 Pro Preview

### All Models

The `AllModels` static reference provides a list of all supported models:

```rust
pub static ref AllModels: Vec<&'static LLMModel> = vec![
    &Sonnet37,
    &Haiku35,
    &Gpt4o,
    &Gpt3oMini,
    &Llama4,
    &QwenQwq32b,
    &QuasarAlpha,
    &Llama4OpenRouter,
    &DeepSeekV3,
    &DeepSeekR1,
    &Gemini25Pro,
];
```

## Adding New Models

To add a new model:

1. Create a cost calculation function:

```rust
fn my_model_cost_calculation(usage: Usage) -> f64 {
    let input_cost = usage.input_tokens as f64 / 1_000_000.0 * 1.00;
    let output_cost = usage.output_tokens as f64 / 1_000_000.0 * 2.00;
    (input_cost + output_cost).max(0.0)
}
```

2. Define the model using the builder pattern:

```rust
lazy_static! {
    pub static ref MyModel: LLMModel = LLMModelBuilder::new()
        .model_name("my-model-id")
        .display_name("My Model")
        .provider(LLMProvider::MyProvider)
        .capabilities(
            ModelCapabilitiesBuilder::new()
                .with_streaming()
                .build()
        )
        .cost_calculation(Arc::new(my_model_cost_calculation))
        .build();
}
```

3. Add the model to the `AllModels` list:

```rust
pub static ref AllModels: Vec<&'static LLMModel> = vec![
    // Existing models...
    &MyModel,
];
```

4. Update the provider's `default_model` method if needed:

```rust
pub fn default_model(&self) -> &'static LLMModel {
    match self {
        // Existing providers...
        LLMProvider::MyProvider => &MyModel,
        _ => panic!("No default model for provider: {}", self.name()),
    }
}
```