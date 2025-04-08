# File System Tools

The file system tools provide operations for reading, writing, creating, and deleting files.

## Overview

File system tools allow the LLM to interact with the local file system, enabling it to read and modify files as needed to complete tasks. These tools are implemented in the `tools/fs` directory.

## Available Tools

### ReadTextFileTool

Reads the content of a text file.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct ReadTextFileParams {
    #[schemars(description = "Relative path to file")]
    pub path: String,
}

#[tool("Read file content")]
pub async fn read_text_file(
    params: ReadTextFileParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "read_text_file",
  "parameters": {
    "path": "src/main.rs"
  }
}
```

### WriteTextFileTool

Writes content to a text file, overwriting any existing content.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct WriteTextFileParams {
    #[schemars(description = "Relative path to file")]
    pub path: String,
    #[schemars(description = "New file content")]
    pub content: String,
}

#[tool("Write file content, overwriting the existing content (use with caution, not for updating part of the file)")]
pub async fn write_text_file(
    params: WriteTextFileParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "write_text_file",
  "parameters": {
    "path": "src/main.rs",
    "content": "fn main() {\n    println!(\"Hello, world!\");\n}"
  }
}
```

### CreateFileTool

Creates an empty file.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct CreateFileParams {
    #[schemars(description = "Relative path to new file")]
    pub path: String,
}

#[tool("Create empty file in given path")]
pub async fn create_file(
    params: CreateFileParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "create_file",
  "parameters": {
    "path": "src/new_file.rs"
  }
}
```

### DeleteFileTool

Deletes a file.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct DeleteFileParams {
    #[schemars(description = "Relative path to file")]
    pub path: String,
}

#[tool("Delete file")]
pub async fn delete_file(
    params: DeleteFileParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "delete_file",
  "parameters": {
    "path": "src/old_file.rs"
  }
}
```

### UpdateTextFilePartialTool

Updates part of a text file based on line numbers or content matching.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct UpdateTextFilePartialParams {
    #[schemars(description = "Relative path to file")]
    pub path: String,
    #[schemars(description = "Start line number (1-based, inclusive)")]
    pub start_line: Option<usize>,
    #[schemars(description = "End line number (1-based, inclusive)")]
    pub end_line: Option<usize>,
    #[schemars(description = "Content to match for start (if start_line not provided)")]
    pub start_pattern: Option<String>,
    #[schemars(description = "Content to match for end (if end_line not provided)")]
    pub end_pattern: Option<String>,
    #[schemars(description = "New content to insert")]
    pub new_content: String,
}

#[tool("Update part of a text file based on line numbers or content matching")]
pub async fn update_text_file_partial(
    params: UpdateTextFilePartialParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "update_text_file_partial",
  "parameters": {
    "path": "src/main.rs",
    "start_line": 5,
    "end_line": 10,
    "new_content": "    // New content\n    println!(\"Hello, world!\");\n"
  }
}
```

### DeleteTextFilePartialTool

Deletes part of a text file based on line numbers or content matching.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct DeleteTextFilePartialParams {
    #[schemars(description = "Relative path to file")]
    pub path: String,
    #[schemars(description = "Start line number (1-based, inclusive)")]
    pub start_line: Option<usize>,
    #[schemars(description = "End line number (1-based, inclusive)")]
    pub end_line: Option<usize>,
    #[schemars(description = "Content to match for start (if start_line not provided)")]
    pub start_pattern: Option<String>,
    #[schemars(description = "Content to match for end (if end_line not provided)")]
    pub end_pattern: Option<String>,
}

#[tool("Delete part of a text file based on line numbers or content matching")]
pub async fn delete_text_file_partial(
    params: DeleteTextFilePartialParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "delete_text_file_partial",
  "parameters": {
    "path": "src/main.rs",
    "start_line": 5,
    "end_line": 10
  }
}
```

## Implementation Details

### Path Safety

All file system tools include safety checks to ensure that paths are:
1. Relative (not absolute)
2. Within the project directory (no path traversal)

This is implemented using the `join_path` utility function:

```rust
pub fn join_path(base: impl Into<PathBuf>, path: impl Into<PathBuf>) -> anyhow::Result<PathBuf> {
    let path = path.into();
    let mut base = base.into();

    if path.is_absolute() {
        return Err(anyhow::anyhow!("Path must be relative"));
    }

    for part in path.iter() {
        if part == OsStr::new("..") {
            if !base.pop() {
                return Err(anyhow::anyhow!("Path exceeds base directory"));
            }
        } else if part != OsStr::new(".") && part != OsStr::new("/") {
            base.push(part);
        }
    }

    Ok(base)
}
```

### Asynchronous I/O

File system operations are implemented using Tokio's asynchronous I/O functions to avoid blocking the main thread:

```rust
use tokio::fs::read_to_string;
use tokio::fs::write;
use tokio::fs::create_dir_all;
use tokio::fs::remove_file;
```

### Error Handling

All tools include comprehensive error handling to provide meaningful error messages to the LLM:

```rust
match file {
    Ok(content) => {
        json!(content)
    }
    Err(e) => json!(e.to_string()),
}
```

## Usage in Nixcode

File system tools are registered in the `Nixcode::new` method:

```rust
tools.add_tool(Arc::new(CreateFileTool {}));
tools.add_tool(Arc::new(ReadTextFileTool {}));
tools.add_tool(Arc::new(WriteTextFileTool {}));
tools.add_tool(Arc::new(DeleteFileTool {}));
// tools.add_tool(Arc::new(UpdateTextFilePartialTool {}));
// tools.add_tool(Arc::new(DeleteTextFilePartialTool {}));
```

Note that some tools are commented out in the current implementation.