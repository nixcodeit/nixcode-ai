# Git Tools

The git tools provide operations for interacting with git repositories, including committing changes, viewing status, managing branches, and more.

## Overview

Git tools allow the LLM to interact with git repositories, enabling it to perform version control operations as needed to complete tasks. These tools are implemented in the `tools/git` directory.

## Available Tools

### GitStatusTool

Gets the current status of the git repository.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitGetTreeProps {}

#[tool("Get git status")]
pub async fn git_status(props: GitGetTreeProps, project: Arc<Project>) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "git_status",
  "parameters": {}
}
```

### GitAddTool

Stages files for commit.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitAddParams {
    #[schemars(description = "Array of files that will be added to index")]
    pub files: Vec<String>,
}

#[tool("Track changes in git")]
pub async fn git_add(params: GitAddParams, project: Arc<Project>) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "git_add",
  "parameters": {
    "files": ["src/main.rs", "src/lib.rs"]
  }
}
```

### GitCommitTool

Commits staged changes.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitCommitProps {
    #[schemars(description = "Message for commit")]
    pub message: String,
}

#[tool("Commit changes")]
pub async fn git_commit(props: GitCommitProps, project: Arc<Project>) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "git_commit",
  "parameters": {
    "message": "Add new feature"
  }
}
```

### GitDiffTool

Shows the differences between the working directory and the index.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitDiffProps {
    #[schemars(description = "Path to the file to show diff for")]
    pub file_path: String,
}

#[tool("Get file diff")]
pub async fn git_diff(props: GitDiffProps, project: Arc<Project>) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "git_diff",
  "parameters": {
    "file_path": "src/main.rs"
  }
}
```

### GitBranchesTool

Lists all branches in the repository.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitBranchesProps {
    #[schemars(description = "Show all branches including remotes (default: false)")]
    pub all: Option<bool>,
}

#[tool("Display git branches")]
pub async fn git_branches(props: GitBranchesProps, project: Arc<Project>) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "git_branches",
  "parameters": {
    "all": true
  }
}
```

### GitBranchCreateTool

Creates a new branch.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitBranchCreateParams {
    #[schemars(description = "Name of the new branch to create")]
    pub branch_name: String,
    #[schemars(description = "Whether to switch to the newly created branch (default: false)")]
    pub switch: Option<bool>,
}

#[tool("Create a new git branch")]
pub async fn git_branch_create(
    params: GitBranchCreateParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "git_branch_create",
  "parameters": {
    "branch_name": "feature/new-feature",
    "switch": true
  }
}
```

### GitBranchDeleteTool

Deletes a branch.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitBranchDeleteParams {
    #[schemars(description = "Name of the branch to delete")]
    pub branch_name: String,
    #[schemars(description = "Force deletion even if branch is not fully merged (default: false)")]
    pub force: Option<bool>,
}

#[tool("Delete a git branch")]
pub async fn git_branch_delete(
    params: GitBranchDeleteParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "git_branch_delete",
  "parameters": {
    "branch_name": "feature/old-feature",
    "force": false
  }
}
```

### GitLogTool

Shows the commit history.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitLogProps {
    #[schemars(description = "Starting reference (commit hash, branch name, or tag)")]
    pub from_ref: Option<String>,
    #[schemars(description = "Ending reference (commit hash, branch name, or tag)")]
    pub to_ref: Option<String>,
    #[schemars(description = "Maximum number of commits to retrieve")]
    pub limit: Option<u32>,
    #[schemars(description = "Path to limit the history to a specific file or directory")]
    pub path: Option<String>,
}

#[tool("Get git log between refs")]
pub async fn git_log(props: GitLogProps, project: Arc<Project>) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "git_log",
  "parameters": {
    "limit": 10
  }
}
```

### GitStashSaveTool

Saves changes to the stash.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitStashSaveParams {
    #[schemars(description = "Optional message describing the stashed changes")]
    pub message: Option<String>,
}

#[tool("Save changes in git stash")]
pub async fn git_stash_save(
    params: GitStashSaveParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "git_stash_save",
  "parameters": {
    "message": "Work in progress"
  }
}
```

### GitStashApplyTool

Applies changes from the stash.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitStashApplyParams {
    #[schemars(description = "Stash index to apply (0 is the most recent stash)")]
    pub stash_index: Option<u32>,
    #[schemars(description = "Whether to drop the stash after applying it")]
    pub pop: Option<bool>,
}

#[tool("Apply changes from git stash")]
pub async fn git_stash_apply(
    params: GitStashApplyParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "git_stash_apply",
  "parameters": {
    "stash_index": 0,
    "pop": true
  }
}
```

### GitStashListTool

Lists all stashes.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitStashListParams {}

#[tool("List all stashes in git repository")]
pub async fn git_stash_list(
    params: GitStashListParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "git_stash_list",
  "parameters": {}
}
```

### GitStashDropTool

Drops a stash.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitStashDropParams {
    #[schemars(description = "Stash index to drop (0 is the most recent stash)")]
    pub stash_index: Option<u32>,
}

#[tool("Drop a stash from git stash list")]
pub async fn git_stash_drop(
    params: GitStashDropParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "git_stash_drop",
  "parameters": {
    "stash_index": 0
  }
}
```

### GitTagCreateTool

Creates a tag.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitTagCreateParams {
    #[schemars(description = "Name of the tag to create")]
    pub tag_name: String,
    #[schemars(description = "Optional message for annotated tag")]
    pub message: Option<String>,
    #[schemars(description = "Commit hash to tag (default: HEAD)")]
    pub target: Option<String>,
    #[schemars(description = "Force overwrite of existing tag (default: false)")]
    pub force: Option<bool>,
}

#[tool("Create a git tag")]
pub async fn git_tag_create(
    params: GitTagCreateParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "git_tag_create",
  "parameters": {
    "tag_name": "v1.0.0",
    "message": "Version 1.0.0"
  }
}
```

### GitTagsListTool

Lists all tags.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitTagsListProps {
    #[schemars(description = "Filter tags by pattern (supports glob patterns)")]
    pub pattern: Option<String>,
    #[schemars(description = "Show tag messages for annotated tags (default: false)")]
    pub show_messages: Option<bool>,
    #[schemars(description = "Sort tags by creation date (default: false - alphabetical)")]
    pub sort_by_date: Option<bool>,
}

#[tool("List git tags")]
pub async fn git_tags_list(props: GitTagsListProps, project: Arc<Project>) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "git_tags_list",
  "parameters": {
    "pattern": "v*",
    "show_messages": true
  }
}
```

## Implementation Details

### Repository Resolution

Git tools use a common utility function to resolve the git repository:

```rust
pub fn resolve_repository(repo_path: Option<PathBuf>) -> Option<Repository> {
    if let Some(path) = repo_path {
        match Repository::open(path) {
            Ok(repo) => Some(repo),
            Err(_) => None,
        }
    } else {
        None
    }
}
```

### Error Handling

All git tools include comprehensive error handling to provide meaningful error messages to the LLM:

```rust
if repository.is_none() {
    return json!("Not a git repository");
}

let repo = repository.unwrap();

let index = repo.index();
if let Err(e) = index {
    return json!(format!("Cannot get index, reason: {}", e));
}
```

## Usage in Nixcode

Git tools are conditionally registered in the `Nixcode::new` method based on whether the project has a git repository:

```rust
if has_repo_path {
    tools.add_tool(Arc::new(GitAddTool {}));
    tools.add_tool(Arc::new(GitCommitTool {}));
    tools.add_tool(Arc::new(GitStatusTool {}));
    tools.add_tool(Arc::new(GitDiffTool {}));
    tools.add_tool(Arc::new(GitStashSaveTool {}));
    tools.add_tool(Arc::new(GitStashApplyTool {}));
    tools.add_tool(Arc::new(GitStashListTool {}));
    tools.add_tool(Arc::new(GitStashDropTool {}));
    tools.add_tool(Arc::new(GitLogTool {}));
    tools.add_tool(Arc::new(GitBranchesTool {}));
    tools.add_tool(Arc::new(GitBranchCreateTool {}));
    tools.add_tool(Arc::new(GitBranchDeleteTool {}));
    tools.add_tool(Arc::new(GitTagCreateTool {}));
    tools.add_tool(Arc::new(GitTagsListTool {}));
}
```