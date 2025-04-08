# Testing Tools Created with nixcode-macros

This guide provides strategies and examples for testing tools created with the `#[tool]` macro from the `nixcode-macros` crate.

## Overview

Testing tools created with the `#[tool]` macro involves testing both:

1. The underlying function that implements the tool's functionality
2. The generated code that implements the `Tool` trait

This guide focuses on both aspects, providing examples and best practices for comprehensive testing.

## Testing the Underlying Function

The underlying function that implements the tool's functionality can be tested like any other Rust function. This is the simplest and most direct way to test your tool's logic.

### Example: Testing a File Reading Tool

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;
    use crate::project::Project;

    #[tokio::test]
    async fn test_read_text_file() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        // Create a test file
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();
        
        // Create a project instance
        let project = Arc::new(Project::new(temp_dir.path().to_path_buf()));
        
        // Create parameters
        let params = ReadFileParams {
            path: "test.txt".to_string(),
        };
        
        // Call the function
        let result = read_text_file(params, project).await;
        
        // Parse the result
        let result_json = result.as_object().unwrap();
        let content = result_json.get("content").unwrap().as_str().unwrap();
        
        // Verify the result
        assert_eq!(content, "Hello, world!\n");
    }
}
```

### Testing Error Handling

It's important to test error cases as well as successful cases:

```rust
#[cfg(test)]
mod tests {
    // ... previous test ...

    #[tokio::test]
    async fn test_read_text_file_nonexistent() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        
        // Create a project instance
        let project = Arc::new(Project::new(temp_dir.path().to_path_buf()));
        
        // Create parameters for a nonexistent file
        let params = ReadFileParams {
            path: "nonexistent.txt".to_string(),
        };
        
        // Call the function
        let result = read_text_file(params, project).await;
        
        // The function should return an error JSON
        let result_json = result.as_object().unwrap();
        
        // Check that the error field exists
        assert!(result_json.contains_key("error"));
    }
}
```

## Testing the Generated Tool Implementation

Testing the generated code that implements the `Tool` trait is more complex, as it involves testing the behavior of the macro itself. However, it's important to ensure that the tool works correctly when invoked through the tool system.

### Example: Testing a Tool Implementation

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Tool;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[tokio::test]
    async fn test_read_text_file_tool() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        // Create a test file
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();
        
        // Create a project instance
        let project = Arc::new(Project::new(temp_dir.path().to_path_buf()));
        
        // Create the tool instance
        let tool = ReadTextFileTool {};
        
        // Create parameters as JSON
        let params = serde_json::json!({
            "path": "test.txt"
        });
        
        // Execute the tool
        let result = tool.execute(params, project).await.unwrap();
        
        // Parse the result
        let result_json = result.as_object().unwrap();
        let content = result_json.get("content").unwrap().as_str().unwrap();
        
        // Verify the result
        assert_eq!(content, "Hello, world!\n");
    }
}
```

### Testing Tool Schema

It's also important to test that the tool's schema is generated correctly:

```rust
#[cfg(test)]
mod tests {
    // ... previous tests ...

    #[test]
    fn test_read_text_file_schema() {
        // Create the tool instance
        let tool = ReadTextFileTool {};
        
        // Get the schema
        let schema = tool.get_schema();
        
        // Verify the schema
        assert_eq!(schema.name, "read_text_file");
        assert_eq!(schema.description, "Read the content of a text file");
        
        // Verify that the schema has the expected properties
        let parameters = schema.parameters.as_object().unwrap();
        let properties = parameters.get("properties").unwrap().as_object().unwrap();
        
        // Check that the "path" property exists
        assert!(properties.contains_key("path"));
    }
}
```

## Integration Testing

Integration tests verify that the tool works correctly when invoked through the tool registry and the LLM client. These tests are more complex but provide the most comprehensive verification of your tool's functionality.

### Example: Integration Test

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::tools::{Tool, ToolRegistry};
    use crate::llm::LLMClient;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[tokio::test]
    async fn test_read_text_file_integration() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        // Create a test file
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();
        
        // Create a project instance
        let project = Arc::new(Project::new(temp_dir.path().to_path_buf()));
        
        // Create a tool registry
        let mut registry = ToolRegistry::new();
        
        // Register the tool
        registry.register(Box::new(ReadTextFileTool {}));
        
        // Create an LLM client (or mock)
        let llm_client = LLMClient::new_mock();
        
        // Simulate an LLM requesting to use the tool
        let tool_request = serde_json::json!({
            "name": "read_text_file",
            "parameters": {
                "path": "test.txt"
            }
        });
        
        // Execute the tool through the registry
        let result = registry.execute_tool(&tool_request, project.clone(), &llm_client).await.unwrap();
        
        // Parse the result
        let result_json = result.as_object().unwrap();
        let content = result_json.get("content").unwrap().as_str().unwrap();
        
        // Verify the result
        assert_eq!(content, "Hello, world!\n");
    }
}
```

## Mocking Dependencies

When testing tools that have external dependencies, it's often useful to mock those dependencies to isolate the tool's functionality.

### Example: Mocking Git Commands

```rust
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct GitStatusParams {}

#[tool("Get git status")]
async fn git_status(params: GitStatusParams, project: Arc<Project>) -> serde_json::Value {
    // In the real implementation, this would call git
    // For testing, we'll use a trait to abstract the git command
    let git = project.git_client();
    let status = git.status()?;
    
    serde_json::json!({ "status": status })
}

// For testing, we can mock the git client
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;

    // Define a trait for the git client
    #[automock]
    trait GitClient {
        fn status(&self) -> anyhow::Result<String>;
    }

    // Implement the trait for the real git client
    struct RealGitClient {
        repo_path: PathBuf,
    }

    impl GitClient for RealGitClient {
        fn status(&self) -> anyhow::Result<String> {
            // Real implementation would call git
            let output = std::process::Command::new("git")
                .current_dir(&self.repo_path)
                .args(&["status"])
                .output()?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Git status failed: {}", error));
            }
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.to_string())
        }
    }

    // Extend the Project struct to provide a git client
    impl Project {
        fn git_client(&self) -> Box<dyn GitClient> {
            Box::new(RealGitClient {
                repo_path: self.root_path().to_path_buf(),
            })
        }
    }

    // For testing, we can create a mock Project that returns a mock git client
    struct MockProject {
        root_path: PathBuf,
        git_client: MockGitClient,
    }

    impl MockProject {
        fn new(root_path: PathBuf) -> Self {
            Self {
                root_path,
                git_client: MockGitClient::new(),
            }
        }
        
        fn root_path(&self) -> &Path {
            &self.root_path
        }
        
        fn git_client(&self) -> Box<dyn GitClient> {
            Box::new(self.git_client.clone())
        }
        
        fn expect_status(&mut self, result: anyhow::Result<String>) {
            self.git_client
                .expect_status()
                .returning(move || result.clone());
        }
    }

    #[tokio::test]
    async fn test_git_status() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        
        // Create a mock project
        let mut mock_project = MockProject::new(temp_dir.path().to_path_buf());
        
        // Set up expectations
        mock_project.expect_status(Ok("On branch main\nNothing to commit\n".to_string()));
        
        // Create parameters
        let params = GitStatusParams {};
        
        // Call the function with the mock project
        let result = git_status(params, Arc::new(mock_project)).await;
        
        // Parse the result
        let result_json = result.as_object().unwrap();
        let status = result_json.get("status").unwrap().as_str().unwrap();
        
        // Verify the result
        assert_eq!(status, "On branch main\nNothing to commit\n");
    }
}
```

## Testing Best Practices

### 1. Test Both Success and Error Cases

Always test both successful execution and error handling. This ensures that your tool behaves correctly in all scenarios.

### 2. Use Temporary Directories

When testing tools that interact with the file system, use temporary directories to avoid affecting the real file system and to ensure test isolation.

### 3. Mock External Dependencies

When testing tools that interact with external systems (like git, databases, or APIs), use mocks to isolate your tests and make them more reliable.

### 4. Test Parameter Validation

Test that your tool correctly validates its parameters and returns appropriate error messages for invalid inputs.

### 5. Test Schema Generation

Verify that your tool's schema is generated correctly, including all required properties and their types.

### 6. Use Integration Tests

In addition to unit tests, use integration tests to verify that your tool works correctly when invoked through the tool registry and the LLM client.

### 7. Test Asynchronous Behavior

Since tools are asynchronous, make sure to test their behavior in an asynchronous context using `tokio::test` or similar.

## Conclusion

Testing tools created with the `#[tool]` macro involves testing both the underlying function and the generated code that implements the `Tool` trait. By following the strategies and examples in this guide, you can ensure that your tools are reliable, robust, and behave correctly in all scenarios.