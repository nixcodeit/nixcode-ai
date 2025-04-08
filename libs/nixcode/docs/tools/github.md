# GitHub Tools

The GitHub tools provide operations for interacting with GitHub repositories, including listing issues and getting issue details.

## Overview

GitHub tools allow the LLM to interact with GitHub repositories, enabling it to access information about issues and other GitHub resources. These tools are implemented in the `tools/github` directory.

## Available Tools

### GithubIssuesListTool

Lists open issues from a GitHub repository.

```rust
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GithubIssuesListParams {
    #[schemars(
        description = "Github account/organization name (default read from config, format: <org>/<repo>)"
    )]
    pub org: Option<String>,

    #[schemars(
        description = "Github repository name (default read from config, format: <org>/<repo>)"
    )]
    pub repo: Option<String>,
}

#[tool("Get list of open issues from GitHub")]
pub async fn github_issues_list(
    params: GithubIssuesListParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "github_issues_list",
  "parameters": {
    "org": "nixcode-ai",
    "repo": "nixcode-ai"
  }
}
```

### GithubIssueDetailsTool

Gets details of a specific issue from a GitHub repository.

```rust
#[derive(JsonSchema, Serialize, Deserialize, Debug)]
pub struct GithubIssueDetailsParams {
    #[schemars(description = "ID of the issue to fetch details for")]
    pub issue_id: u64,

    #[schemars(
        description = "Github account/organization name (default read from config, format: <org>/<repo>)"
    )]
    pub org: Option<String>,

    #[schemars(
        description = "Github repository name (default read from config, format: <org>/<repo>)"
    )]
    pub repo: Option<String>,
}

#[tool("Get details of issue from GitHub")]
pub async fn github_issue_details(
    params: GithubIssueDetailsParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

Example usage:
```json
{
  "name": "github_issue_details",
  "parameters": {
    "issue_id": 42,
    "org": "nixcode-ai",
    "repo": "nixcode-ai"
  }
}
```

## Implementation Details

### GitHub API Integration

GitHub tools use the `octocrab` crate to interact with the GitHub API:

```rust
let client = octocrab::instance();
let issue = client.issues(org, repo).get(params.issue_id).await;
```

### Authentication

GitHub API authentication is handled in the `Nixcode::new_with_config` method:

```rust
if let Ok(token) = config.get_github_token() {
    let client = octocrab::OctocrabBuilder::new()
        .user_access_token(token.clone())
        .build();

    if let Err(e) = client {
        log::error!("Failed to initialize Octocrab client: {}", e);

        return Err(LLMError::Generic(format!(
            "Failed to initialize Octocrab client: {}",
            e
        )));
    }

    octocrab::initialise(client.unwrap());
}
```

### Repository Information

GitHub tools use repository information from the project context or from the parameters:

```rust
let project_github = project.get_github();
let org = params
    .org
    .or_else(|| project_github.clone().and_then(|p| p.org));
let repo = params
    .repo
    .or_else(|| project_github.clone().and_then(|p| p.repo));
if org.is_none() || repo.is_none() {
    return json!("GitHub organization or repository not specified");
}
```

### Error Handling

GitHub tools include comprehensive error handling to provide meaningful error messages to the LLM:

```rust
if let Err(e) = issue {
    return json!(format!("Error fetching issue: {}", e));
}
```

## Usage in Nixcode

GitHub tools are registered in the `Nixcode::new` method:

```rust
// GitHub tools
tools.add_tool(Arc::new(GithubIssuesListTool {}));
tools.add_tool(Arc::new(GithubIssueDetailsTool {}));
```

## Configuration

GitHub integration requires configuration in the `config.toml` file:

```toml
[github]
token = "${GITHUB_TOKEN}"
org = "nixcode-ai"
repo = "nixcode-ai"
```

The token can be specified directly or using environment variable expansion with the `${VAR_NAME}` syntax.