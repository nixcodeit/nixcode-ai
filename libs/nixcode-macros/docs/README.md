# nixcode-macros Documentation

## Overview

`nixcode-macros` is a procedural macro crate that provides custom attribute macros for the nixcode-ai project. The primary purpose of this crate is to simplify the implementation of tools that can be invoked by Large Language Models (LLMs) within the nixcode-ai application.

Currently, the crate provides a single procedural attribute macro: `#[tool]`, which automates the boilerplate code required to implement the `Tool` trait for functions that provide functionality to LLMs.

## Installation

The `nixcode-macros` crate is part of the nixcode-ai workspace. To use it in another crate within the workspace, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
nixcode-macros = { path = "../nixcode-macros" }
```

## Dependencies

The crate relies on the following dependencies:

- `syn` (version 2.0 with "full" and "parsing" features): For parsing Rust code
- `quote` (version 1.0): For generating Rust code
- `proc-macro2` (version 1.0): For working with procedural macros
- `schemars` (version 0.8): For generating JSON Schema
- `serde_json` (version 1.0): For working with JSON data

## Macros

### `#[tool]` Attribute Macro

The `#[tool]` attribute macro is designed to be applied to async functions that implement tool functionality for LLMs. It automatically generates the boilerplate code needed to implement the `Tool` trait, which is required for the function to be usable as a tool by LLMs in the nixcode-ai application.

#### Usage

```rust
use nixcode_macros::tool;

#[tool("Description of what this tool does")]
async fn my_tool(params: MyToolParams, project: std::sync::Arc<Project>) -> serde_json::Value {
    // Tool implementation
    // ...
    serde_json::json!({ "result": "success" })
}
```

#### Parameters

The `#[tool]` macro accepts an optional string literal parameter that provides a description of the tool. If no description is provided, a default description is generated based on the function name.

#### Requirements

Functions annotated with the `#[tool]` macro must:

1. Be `async` functions
2. Have at least one parameter
3. The first parameter must be a struct that can be deserialized from JSON (typically using `serde`)
4. The second parameter must be of type `std::sync::Arc<crate::project::Project>`
5. Return a value that can be serialized to JSON (typically `serde_json::Value`)

#### Generated Code

For a function annotated with `#[tool]`, the macro generates:

1. A struct with a name derived from the function name (converted to PascalCase with "Tool" appended)
2. An implementation of the `Tool` trait for this struct, which:
   - Returns the function name as the tool name
   - Generates a JSON Schema for the function's parameter type
   - Executes the function with deserialized parameters when the tool is invoked

#### Example

Given the following function:

```rust
#[tool("Search for files matching a glob pattern")]
async fn search_glob_files(params: GlobToolParams, project: std::sync::Arc<Project>) -> serde_json::Value {
    // Implementation...
    serde_json::json!({ "files": ["file1.txt", "file2.txt"] })
}
```

The macro will generate code equivalent to:

```rust
pub struct SearchGlobFilesTool {}

#[async_trait::async_trait]
impl crate::tools::Tool for SearchGlobFilesTool {
    fn get_name(&self) -> String {
        "search_glob_files".to_string()
    }

    fn get_schema(&self) -> nixcode_llm_sdk::tools::Tool {
        let schema = schemars::schema_for!(GlobToolParams);
        let mut parameters = serde_json::to_value(&schema).unwrap();
        let mut obj = parameters.as_object_mut().unwrap();
        if !obj.contains_key("properties") {
            obj.extend([
                ("properties".into(), serde_json::json!({}))
            ]);
        }

        let tool_name = "search_glob_files".to_string();
        let description = "Search for files matching a glob pattern".to_string();

        nixcode_llm_sdk::tools::Tool::new(tool_name, description, parameters)
    }

    async fn execute(&self, params: serde_json::Value, project: std::sync::Arc<crate::project::Project>) -> anyhow::Result<serde_json::Value> {
        let params: GlobToolParams = serde_json::from_value(params)?;
        Ok(search_glob_files(params, project).await)
    }
}
```

## Implementation Details

### Function Name Transformation

The macro converts the snake_case function name to PascalCase and appends "Tool" to create the struct name. For example:
- `search_glob_files` becomes `SearchGlobFilesTool`
- `git_commit` becomes `GitCommitTool`

### Parameter Type Extraction

The macro extracts the type of the first parameter of the function to use it for deserialization of the tool parameters. It expects this parameter to be a struct that can be deserialized from JSON.

### JSON Schema Generation

The macro uses `schemars::schema_for!` to generate a JSON Schema for the parameter type. This schema is used to validate and document the parameters that the tool accepts.

### Error Handling

The macro includes compile-time error checks for:
- Functions with no parameters
- Parameters that are not properly typed

## Best Practices

1. **Parameter Types**: Create a dedicated struct for your tool's parameters, even if it only has one field. This makes it easier to add more parameters in the future without breaking changes.

2. **Documentation**: Always provide a clear description for your tool using the string literal parameter of the `#[tool]` macro.

3. **Error Handling**: Use `anyhow::Result` for comprehensive error handling within your tool implementation.

4. **Parameter Validation**: Implement validation for your parameters within your tool function to ensure they meet your requirements.

5. **Return Values**: Return structured JSON data that can be easily processed by the LLM.

## Example Use Cases

### File System Tool

```rust
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct ReadFileParams {
    pub path: String,
}

#[tool("Read the content of a text file")]
async fn read_text_file(params: ReadFileParams, project: std::sync::Arc<Project>) -> serde_json::Value {
    let content = std::fs::read_to_string(&params.path)
        .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;
    
    serde_json::json!({ "content": content })
}
```

### Git Tool

```rust
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct GitCommitParams {
    pub message: String,
}

#[tool("Commit changes to the git repository")]
async fn git_commit(params: GitCommitParams, project: std::sync::Arc<Project>) -> serde_json::Value {
    // Implementation...
    serde_json::json!({ "success": true, "commit_hash": "abc123" })
}
```

## Limitations

1. The `#[tool]` macro currently only supports functions with a specific signature. It requires:
   - An async function
   - A first parameter that can be deserialized from JSON
   - A second parameter of type `std::sync::Arc<crate::project::Project>`
   - A return type that can be serialized to JSON

2. The macro does not support functions with generic parameters or lifetimes.

3. The macro does not support functions with variadic arguments.

## Future Improvements

Potential future improvements to the `nixcode-macros` crate could include:

1. Support for more flexible function signatures
2. Additional macros for other common patterns in the nixcode-ai project
3. Better error messages for common mistakes
4. Support for generating documentation from the tool descriptions
5. Support for tool categories or grouping

## Internal Implementation

The `#[tool]` macro works by:

1. Parsing the function definition using `syn`
2. Extracting the function name, parameters, and return type
3. Generating a new struct with a name derived from the function name
4. Implementing the `Tool` trait for this struct
5. Using the original function in the `execute` method of the trait implementation

The generated code uses `serde_json` for serialization and deserialization, `schemars` for JSON Schema generation, and `anyhow` for error handling.

## Conclusion

The `nixcode-macros` crate provides a convenient way to implement tools for LLMs in the nixcode-ai project. By using the `#[tool]` attribute macro, developers can focus on implementing the tool's functionality without having to write boilerplate code for the `Tool` trait implementation.