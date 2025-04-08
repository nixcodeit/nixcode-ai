# nixcode-macros Documentation

Welcome to the documentation for the `nixcode-macros` crate, a collection of procedural macros designed to simplify development in the nixcode-ai project.

## Table of Contents

1. [Overview](README.md) - Introduction and high-level overview of the crate
2. [Technical Documentation](technical.md) - Detailed technical explanation of the implementation
3. [Usage Guide](usage-guide.md) - Practical examples and best practices
4. [Testing Guide](testing.md) - Strategies and examples for testing tools

## Quick Start

The `nixcode-macros` crate currently provides a single procedural attribute macro: `#[tool]`, which automates the boilerplate code required to implement the `Tool` trait for functions that provide functionality to LLMs.

### Basic Example

```rust
use nixcode_macros::tool;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;
use crate::project::Project;

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct MyToolParams {
    pub input_text: String,
}

#[tool("A description of what my tool does")]
async fn my_tool(params: MyToolParams, project: Arc<Project>) -> serde_json::Value {
    // Tool implementation
    let result = format!("Processed: {}", params.input_text);
    
    serde_json::json!({
        "result": result
    })
}
```

## Key Features

- **Automatic Tool Trait Implementation**: The `#[tool]` macro automatically implements the `Tool` trait for your function, eliminating boilerplate code.
- **JSON Schema Generation**: The macro generates a JSON Schema for your tool's parameters, which is used for validation and documentation.
- **Error Checking**: The macro includes compile-time error checks to ensure your tool is implemented correctly.
- **Customizable Descriptions**: You can provide a custom description for your tool, which is used by the LLM to understand what the tool does.

## Documentation Structure

The documentation is organized into four main sections:

1. **Overview**: Provides a high-level introduction to the crate, its purpose, and its features.
2. **Technical Documentation**: Offers a detailed explanation of the internal implementation of the macros.
3. **Usage Guide**: Presents practical examples and best practices for using the macros effectively.
4. **Testing Guide**: Provides strategies and examples for testing tools created with the macros.

Each section is designed to be read independently, but together they provide a comprehensive understanding of the crate.

## Contributing

If you'd like to contribute to the `nixcode-macros` crate, please follow these guidelines:

1. Ensure that any new macros follow the same pattern as the existing ones.
2. Add comprehensive documentation for any new features.
3. Include tests to verify that the macros work as expected.
4. Follow the Rust coding guidelines and use the standard formatting.

## License

The `nixcode-macros` crate is part of the nixcode-ai project and is subject to the same license terms.

## Contact

If you have any questions or feedback about the `nixcode-macros` crate, please open an issue on the nixcode-ai GitHub repository.