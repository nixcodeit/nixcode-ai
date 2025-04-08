# Project

The project module provides functionality for managing project context, including the current working directory, git repository information, and project analysis.

## Overview

The `Project` struct represents the context of the current project, including:
- The current working directory
- The git repository path (if available)
- Project initialization analysis content (if available)
- GitHub repository information (if available)

This context is used by tools to operate in the correct environment and provide relevant information to the LLM.

## Project Structure

```rust
#[derive(Clone, Debug)]
pub struct Project {
    cwd: PathBuf,
    project_init_analysis_content: Option<String>,
    repo_path: Option<PathBuf>,
    github: Option<GitHub>,
}
```

### Fields

- `cwd`: The current working directory
- `project_init_analysis_content`: The content of the project initialization analysis (from `.nixcode/init.md`)
- `repo_path`: The path to the git repository (if available)
- `github`: GitHub repository information (if available)

## GitHub Structure

```rust
#[derive(Clone, Debug)]
pub struct GitHub {
    /// GitHub account/organization name
    pub org: Option<String>,

    /// GitHub repository name
    pub repo: Option<String>,
}
```

### Fields

- `org`: The GitHub organization or account name
- `repo`: The GitHub repository name

## Project Methods

### Initialization

```rust
pub fn new(cwd: PathBuf) -> Self
```

Creates a new `Project` instance with the provided current working directory. This method:
1. Checks for the existence of `.nixcode/init.md` and loads its content if available
2. Attempts to discover a git repository in the current directory or its parents
3. Initializes the `Project` struct with the discovered information

### Accessors

```rust
pub fn get_cwd(&self) -> PathBuf
```

Returns the current working directory.

```rust
pub fn get_project_init_analysis_content(&self) -> Option<String>
```

Returns the content of the project initialization analysis, if available.

```rust
pub fn has_init_analysis(&self) -> bool
```

Returns whether the project has an initialization analysis.

```rust
pub fn has_repo_path(&self) -> bool
```

Returns whether the project has a git repository.

```rust
pub fn get_repo_path(&self) -> Option<PathBuf>
```

Returns the path to the git repository, if available.

### GitHub Integration

```rust
pub fn set_github(&mut self, github: &GitHubSettings) -> &mut Self
```

Sets the GitHub repository information from the provided settings.

```rust
pub fn get_github(&self) -> Option<GitHub>
```

Returns the GitHub repository information, if available.

## Project Initialization Analysis

The project initialization analysis is a Markdown document stored in `.nixcode/init.md` that provides a comprehensive overview of the project structure, architecture, and organization. This analysis is used by the LLM to understand the project context and provide more relevant assistance.

The analysis is generated using the `get_project_analysis_prompt` tool, which provides a prompt for the LLM to analyze the project and generate the analysis document.

## Example Usage

```rust
// Create a new project with the current directory
let project = Project::new(std::env::current_dir().unwrap());

// Check if the project has a git repository
if project.has_repo_path() {
    println!("Git repository found at: {:?}", project.get_repo_path().unwrap());
}

// Check if the project has an initialization analysis
if project.has_init_analysis() {
    println!("Project analysis found");
}

// Set GitHub repository information
let mut project = Project::new(std::env::current_dir().unwrap());
project.set_github(&GitHubSettings {
    org: Some("nixcode-ai".to_string()),
    repo: Some("nixcode-ai".to_string()),
    token: None,
});
```