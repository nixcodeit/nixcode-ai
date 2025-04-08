# Usage Guide: nixcode-macros

This guide provides practical examples and best practices for using the `#[tool]` macro from the `nixcode-macros` crate to implement tools for LLMs in the nixcode-ai project.

## Basic Usage

### Step 1: Define Parameter Struct

First, define a struct for your tool's parameters. This struct should be serializable and deserializable with `serde`, and should have a JSON schema defined with `schemars`.

```rust
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct MyToolParams {
    pub input_text: String,
    #[serde(default)]
    pub optional_flag: bool,
}
```

### Step 2: Implement Tool Function

Next, implement the function that will be executed when the tool is invoked. This function should be annotated with the `#[tool]` macro.

```rust
use nixcode_macros::tool;
use std::sync::Arc;
use crate::project::Project;

#[tool("A description of what my tool does")]
async fn my_tool(params: MyToolParams, project: Arc<Project>) -> serde_json::Value {
    // Tool implementation
    let result = format!("Processed: {}", params.input_text);
    
    serde_json::json!({
        "result": result,
        "flag_was_set": params.optional_flag
    })
}
```

### Step 3: Register Tool

Finally, register the tool with the tool registry in your application.

```rust
use crate::tools::{Tool, ToolRegistry};

pub fn register_tools(registry: &mut ToolRegistry) {
    registry.register(Box::new(MyToolTool {}));
}
```

## Advanced Usage

### Parameter Validation

You can add validation to your tool parameters by implementing custom deserialization logic:

```rust
use serde::{Deserialize, Serialize, Deserializer};
use schemars::JsonSchema;
use anyhow::{Result, anyhow};

#[derive(Serialize, JsonSchema)]
pub struct PathParams {
    pub path: String,
}

impl<'de> Deserialize<'de> for PathParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawParams {
            path: String,
        }

        let raw = RawParams::deserialize(deserializer)?;
        
        // Validate path
        if raw.path.contains("..") {
            return Err(serde::de::Error::custom("Path cannot contain '..'"));
        }
        
        Ok(PathParams {
            path: raw.path,
        })
    }
}

#[tool("Read a file with path validation")]
async fn read_validated_file(params: PathParams, project: Arc<Project>) -> serde_json::Value {
    // Implementation
}
```

### Nested Parameters

You can use nested structures for more complex parameter sets:

```rust
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub regex: bool,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct SearchParams {
    pub pattern: String,
    pub file_glob: String,
    #[serde(default)]
    pub options: SearchOptions,
}

#[tool("Search for text in files with advanced options")]
async fn advanced_search(params: SearchParams, project: Arc<Project>) -> serde_json::Value {
    // Implementation
}
```

### Error Handling

Proper error handling is important for tools. Use `anyhow::Result` for comprehensive error handling:

```rust
#[tool("A tool with proper error handling")]
async fn robust_tool(params: MyToolParams, project: Arc<Project>) -> serde_json::Value {
    // Try to perform an operation that might fail
    let result = std::fs::read_to_string(&params.input_text)
        .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;
    
    // Process the result
    let processed = process_data(&result)
        .map_err(|e| anyhow::anyhow!("Failed to process data: {}", e))?;
    
    serde_json::json!({
        "success": true,
        "result": processed
    })
}

fn process_data(data: &str) -> Result<String, std::io::Error> {
    // Process the data
    Ok(data.to_uppercase())
}
```

### Optional Parameters

You can make parameters optional using `Option<T>` or default values with `#[serde(default)]`:

```rust
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct OptionalParams {
    pub required_param: String,
    pub optional_param: Option<String>,
    #[serde(default)]
    pub param_with_default: bool,
}

#[tool("A tool with optional parameters")]
async fn optional_params_tool(params: OptionalParams, project: Arc<Project>) -> serde_json::Value {
    let optional_value = params.optional_param.unwrap_or_else(|| "default".to_string());
    
    serde_json::json!({
        "required": params.required_param,
        "optional": optional_value,
        "default": params.param_with_default
    })
}
```

### Enum Parameters

You can use enums for parameters with a fixed set of possible values:

```rust
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct SortParams {
    pub field: String,
    #[serde(default = "default_sort_order")]
    pub order: SortOrder,
}

fn default_sort_order() -> SortOrder {
    SortOrder::Ascending
}

#[tool("Sort data by field")]
async fn sort_data(params: SortParams, project: Arc<Project>) -> serde_json::Value {
    // Implementation
}
```

### Returning Structured Data

Return structured data that can be easily processed by the LLM:

```rust
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct FileInfoParams {
    pub path: String,
}

#[tool("Get file information")]
async fn file_info(params: FileInfoParams, project: Arc<Project>) -> serde_json::Value {
    let metadata = std::fs::metadata(&params.path)
        .map_err(|e| anyhow::anyhow!("Failed to get metadata: {}", e))?;
    
    serde_json::json!({
        "exists": true,
        "is_file": metadata.is_file(),
        "is_dir": metadata.is_dir(),
        "size_bytes": metadata.len(),
        "readonly": metadata.permissions().readonly(),
        "modified": metadata.modified().ok().map(|time| {
            time.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        }),
    })
}
```

## Real-World Examples

### File System Tool

```rust
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct ReadFileParams {
    pub path: String,
}

#[tool("Read the content of a text file")]
async fn read_text_file(params: ReadFileParams, project: Arc<Project>) -> serde_json::Value {
    let path = project.resolve_path(&params.path)?;
    
    let content = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", path.display(), e))?;
    
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
async fn git_commit(params: GitCommitParams, project: Arc<Project>) -> serde_json::Value {
    let repo_path = project.root_path();
    
    // Create git command
    let output = std::process::Command::new("git")
        .current_dir(repo_path)
        .args(&["commit", "-m", &params.message])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute git command: {}", e))?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return serde_json::json!({
            "success": false,
            "error": error.to_string()
        });
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    serde_json::json!({
        "success": true,
        "message": stdout.to_string()
    })
}
```

### Search Tool

```rust
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct SearchContentParams {
    pub pattern: String,
    pub glob_pattern: String,
    #[serde(default)]
    pub include_hidden: bool,
    #[serde(default)]
    pub include_gitignored: bool,
}

#[tool("Search for text content in files using regex pattern")]
async fn search_content(params: SearchContentParams, project: Arc<Project>) -> serde_json::Value {
    // Implementation
    serde_json::json!({
        "matches": [
            {
                "file": "src/main.rs",
                "line": 10,
                "content": "    let pattern = \"example\";"
            }
        ]
    })
}
```

## Best Practices

### 1. Clear Descriptions

Always provide a clear and concise description for your tool using the string literal parameter of the `#[tool]` macro. This description will be used by the LLM to understand what the tool does.

```rust
#[tool("Convert a string to uppercase")]  // Good
async fn to_uppercase(params: TextParams, project: Arc<Project>) -> serde_json::Value {
    // Implementation
}

#[tool]  // Not recommended - uses auto-generated description
async fn to_uppercase(params: TextParams, project: Arc<Project>) -> serde_json::Value {
    // Implementation
}
```

### 2. Dedicated Parameter Types

Create a dedicated struct for your tool's parameters, even if it only has one field. This makes it easier to add more parameters in the future without breaking changes.

```rust
// Good - dedicated parameter type
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct UppercaseParams {
    pub text: String,
}

#[tool("Convert a string to uppercase")]
async fn to_uppercase(params: UppercaseParams, project: Arc<Project>) -> serde_json::Value {
    // Implementation
}
```

### 3. Comprehensive Error Handling

Use `anyhow::Result` for comprehensive error handling within your tool implementation. Provide clear error messages that explain what went wrong.

```rust
#[tool("Read a file")]
async fn read_file(params: ReadFileParams, project: Arc<Project>) -> serde_json::Value {
    // Good - detailed error message
    let content = std::fs::read_to_string(&params.path)
        .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", params.path, e))?;
    
    serde_json::json!({ "content": content })
}
```

### 4. Parameter Validation

Implement validation for your parameters within your tool function to ensure they meet your requirements.

```rust
#[tool("Create a new file")]
async fn create_file(params: CreateFileParams, project: Arc<Project>) -> serde_json::Value {
    // Good - validate parameters
    if params.path.contains("..") {
        return serde_json::json!({
            "success": false,
            "error": "Path cannot contain '..'"
        });
    }
    
    // Implementation
}
```

### 5. Structured Return Values

Return structured JSON data that can be easily processed by the LLM. Include a `success` field to indicate whether the operation was successful.

```rust
#[tool("Delete a file")]
async fn delete_file(params: DeleteFileParams, project: Arc<Project>) -> serde_json::Value {
    // Good - structured return value
    match std::fs::remove_file(&params.path) {
        Ok(_) => serde_json::json!({
            "success": true,
            "message": format!("File '{}' deleted successfully", params.path)
        }),
        Err(e) => serde_json::json!({
            "success": false,
            "error": format!("Failed to delete file '{}': {}", params.path, e)
        }),
    }
}
```

### 6. Documentation

Document your tool's parameters and return values to make it easier for other developers to understand how to use your tool.

```rust
/// A tool for searching files by glob pattern
///
/// # Parameters
///
/// * `pattern` - Glob pattern to match files (e.g., "**/*.rs")
/// * `include_hidden` - Whether to include hidden files (default: false)
/// * `include_gitignored` - Whether to include files excluded by gitignore (default: false)
///
/// # Returns
///
/// A JSON object with a `files` array containing the paths of matching files.
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct GlobToolParams {
    pub pattern: String,
    #[serde(default)]
    pub include_hidden: bool,
    #[serde(default)]
    pub include_gitignored: bool,
}

#[tool("Search for files matching a glob pattern")]
async fn search_glob_files(params: GlobToolParams, project: Arc<Project>) -> serde_json::Value {
    // Implementation
}
```

## Conclusion

The `#[tool]` macro from the `nixcode-macros` crate provides a convenient way to implement tools for LLMs in the nixcode-ai project. By following the guidelines and examples in this usage guide, you can create effective and robust tools that enhance the capabilities of the LLMs in your application.