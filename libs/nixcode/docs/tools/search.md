# Search Tools

The search tools provide operations for searching and replacing content in files.

## Overview

Search tools allow the LLM to search for content in files and replace content in files, enabling it to find and modify code as needed to complete tasks. These tools are implemented in the `tools/search` directory.

## Available Tools

### SearchContentTool

Searches for text content in files using a regex pattern.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct SearchContentParams {
    #[schemars(description = "Regex pattern to search for in file content")]
    pub pattern: String,

    #[schemars(description = "Glob pattern for files to search in")]
    pub glob_pattern: String,

    #[schemars(description = "Include files excluded by gitignore (default: false)")]
    #[serde(default)]
    pub include_gitignored: Option<bool>,

    #[schemars(
        description = "Include hidden (prefixed with `.`, like `.github`, `.nixcode` etc) (default: false)"
    )]
    #[serde(default)]
    pub include_hidden: Option<bool>,

    #[schemars(description = "Offset for search results (default: 0)")]
    #[serde(default)]
    pub offset: Option<usize>,
}

#[tool("Search for text content in files using regex pattern")]
pub async fn search_content(
    params: SearchContentParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "search_content",
  "parameters": {
    "pattern": "fn main\\(\\)",
    "glob_pattern": "src/**/*.rs"
  }
}
```

### ReplaceContentTool

Replaces text content in files based on a regex pattern.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct ReplaceContentParams {
    #[schemars(description = "Regex pattern to search for in file content")]
    pub pattern: String,

    #[schemars(description = "Replacement string (can use regex capture groups like $1, $2)")]
    pub replacement: String,

    #[schemars(description = "Glob pattern for files to search in")]
    pub glob_pattern: String,

    #[schemars(description = "Include files excluded by gitignore (default: false)")]
    #[serde(default)]
    pub include_gitignored: Option<bool>,

    #[schemars(
        description = "Include hidden (prefixed with `.`, like `.github`, `.nixcode` etc) (default: false)"
    )]
    #[serde(default)]
    pub include_hidden: Option<bool>,
}

#[tool("Replace text content in files based on regex pattern")]
pub async fn replace_content(
    params: ReplaceContentParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "replace_content",
  "parameters": {
    "pattern": "fn main\\(\\) \\{",
    "replacement": "fn main() {\n    // Added comment",
    "glob_pattern": "src/**/*.rs"
  }
}
```

## Implementation Details

### Content Utilities

Search tools use common utility functions for validating regex patterns, resolving glob patterns, and filtering paths:

```rust
pub fn validate_regex(pattern: &str) -> Result<Regex, serde_json::Value> {
    match Regex::new(pattern) {
        Ok(re) => Ok(re),
        Err(e) => Err(json!(format!("Invalid regex pattern: {}", e))),
    }
}

pub fn validate_and_resolve_glob(
    project: &Arc<Project>,
    glob_pattern: &str,
) -> Result<String, serde_json::Value> {
    let cwd = project.get_cwd();
    let cwd_str = cwd.to_string_lossy();

    // Ensure the glob pattern is relative
    if glob_pattern.starts_with('/') {
        return Err(json!("Glob pattern must be relative"));
    }

    // Resolve the glob pattern to an absolute path
    let pattern_str = format!("{}/{}", cwd_str, glob_pattern);
    Ok(pattern_str)
}

pub async fn get_glob_paths(
    pattern: String,
) -> Result<Vec<PathBuf>, serde_json::Value> {
    let paths = tokio::task::spawn_blocking(move || -> Result<Vec<PathBuf>, glob::PatternError> {
        let mut paths = Vec::new();
        for entry in glob::glob(&pattern)? {
            if let Ok(path) = entry {
                paths.push(path);
            }
        }
        Ok(paths)
    })
    .await;

    match paths {
        Ok(Ok(paths)) => Ok(paths),
        Ok(Err(e)) => Err(json!(format!("Invalid glob pattern: {}", e))),
        Err(e) => Err(json!(format!("Task error: {}", e))),
    }
}

pub async fn filter_paths(
    project: Arc<Project>,
    paths: Vec<PathBuf>,
    include_hidden: bool,
    include_git: bool,
) -> Vec<(PathBuf, String)> {
    // Implementation
}
```

### Asynchronous Processing

Search operations are performed asynchronously using Tokio's `spawn_blocking` to avoid blocking the main thread:

```rust
let regex_pattern = regex.clone();
let results = tokio::task::spawn_blocking(move || {
    let mut matches = Vec::new();
    let mut total_matches = 0;

    for (file_path, rel_path) in filtered_paths {
        // Implementation
    }

    (matches, total_matches)
})
.await
.unwrap();
```

### Result Formatting

Search results are formatted to provide clear and concise information to the LLM:

```rust
if results.0.is_empty() {
    json!("No matches found")
} else {
    let (matches, total_matches) = results;
    let mut result_str = format!(
        "Found {} matches for pattern '{}' in files matching '{}':\n\n",
        total_matches, params.pattern, params.glob_pattern
    );

    for m in &matches {
        result_str.push_str(&format!(
            "{}:{}: {}\n",
            m.path, m.line_number, m.line_content
        ));
    }

    let missing_results = total_matches.saturating_sub(offset + matches.len());
    if missing_results > 0 {
        if offset > 0 {
            result_str.push_str(&format!("\n... and {} more matches (current offset: {}), reuse tool with offset parameter", missing_results, offset));
        } else {
            result_str.push_str(&format!(
                "\n... and {} more matches, reuse tool with offset parameter",
                missing_results
            ));
        }
    }

    json!(result_str)
}
```

## Usage in Nixcode

Search tools are registered in the `Nixcode::new` method:

```rust
tools.add_tool(Arc::new(SearchContentTool {}));
tools.add_tool(Arc::new(ReplaceContentTool {}));
```