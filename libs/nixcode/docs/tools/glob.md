# Glob Tools

The glob tools provide operations for searching for files using glob patterns.

## Overview

Glob tools allow the LLM to search for files using glob patterns, enabling it to find files that match specific patterns. These tools are implemented in the `tools/glob` directory.

## Available Tools

### SearchGlobFilesTool

Searches for files in the project directory using a glob pattern.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GlobToolParams {
    #[schemars(description = "Glob pattern")]
    pub pattern: String,

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

#[tool("Search for files in project directory using glob pattern")]
pub async fn search_glob_files(
    params: GlobToolParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "search_glob_files",
  "parameters": {
    "pattern": "src/**/*.rs"
  }
}
```

## Implementation Details

### Glob Pattern Resolution

Glob patterns are resolved to absolute paths using the project's current working directory:

```rust
let cwd = project.get_cwd();
let cwd_str = cwd.to_string_lossy();

// Ensure the glob pattern is relative
if params.pattern.starts_with('/') {
    return json!("Glob pattern must be relative");
}

// Resolve the glob pattern to an absolute path
let pattern_str = format!("{}/{}", cwd_str, params.pattern);
```

### Glob Matching

Glob matching is performed using the `glob` crate:

```rust
let paths = tokio::task::spawn_blocking(move || -> Result<Vec<PathBuf>, glob::PatternError> {
    let mut paths = Vec::new();
    for entry in glob::glob(&pattern_str)? {
        if let Ok(path) = entry {
            paths.push(path);
        }
    }
    Ok(paths)
})
.await;
```

### Path Filtering

Paths are filtered based on the `include_hidden` and `include_gitignored` parameters:

```rust
// Filter out hidden files/directories if not included
if !include_hidden && is_hidden_path(&path) {
    continue;
}

// Filter out gitignored files if not included
if !include_git {
    let is_ignored = gitignore::is_path_ignored(&path);
    if is_ignored {
        continue;
    }
}
```

### Result Formatting

Glob search results are formatted to provide clear and concise information to the LLM:

```rust
if filtered_paths.is_empty() {
    json!("No files found matching the pattern")
} else {
    let mut result_str = format!(
        "Found {} files matching pattern '{}':\n\n",
        filtered_paths.len(),
        params.pattern
    );

    for path in &filtered_paths[offset..end] {
        result_str.push_str(&format!("{}\n", path));
    }

    if end < filtered_paths.len() {
        result_str.push_str(&format!(
            "\n... and {} more files (showing {} to {}), reuse tool with offset parameter",
            filtered_paths.len() - end,
            offset + 1,
            end
        ));
    }

    json!(result_str)
}
```

## Usage in Nixcode

Glob tools are registered in the `Nixcode::new` method:

```rust
tools.add_tool(Arc::new(SearchGlobFilesTool {}));
```

## Testing

The glob tools include tests to verify their functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Project;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_search_glob_files() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // Create some test files
        create_test_file(&temp_path.join("file1.txt"), "test content").await;
        create_test_file(&temp_path.join("file2.txt"), "test content").await;
        create_test_file(&temp_path.join("subdir/file3.txt"), "test content").await;

        // Create a project with the temporary directory
        let project = Project::new(temp_path.clone());

        // Test the search_glob_files function
        let params = GlobToolParams {
            pattern: "**/*.txt".to_string(),
            include_gitignored: None,
            include_hidden: None,
            offset: None,
        };

        let result = search_glob_files(params, Arc::new(project)).await;
        let result_str = result.as_str().unwrap();

        // Verify the results
        assert!(result_str.contains("Found"));
        assert!(result_str.contains("file1.txt"));
        assert!(result_str.contains("file2.txt"));
        assert!(result_str.contains("subdir/file3.txt"));
    }

    async fn create_test_file(path: &PathBuf, content: &str) {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.unwrap();
        }

        // Create and write to the file
        let mut file = File::create(path).await.unwrap();
        file.write_all(content.as_bytes()).await.unwrap();
    }
}
```