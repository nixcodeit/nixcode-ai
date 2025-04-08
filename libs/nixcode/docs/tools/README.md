# Tools

The tools module provides a framework for LLM function calling, allowing the LLM to execute various operations on the local system.

## Overview

The tools system is a key architectural feature of the nixcode-ai application, enabling the LLM to:
1. Request tool use through a standardized JSON interface
2. Have tools executed by the application
3. Receive results back to continue the conversation

This enables capabilities like file searching, content reading/writing, git operations, cargo commands, and more.

## Tool Categories

The tools are organized into several categories:

- [File System Tools](./fs.md): Operations on files and directories
- [Git Tools](./git.md): Git repository operations
- [GitHub Tools](./github.md): GitHub API integration
- [Search Tools](./search.md): Content searching and replacement
- [Glob Tools](./glob.md): File pattern matching
- [Command Tools](./commands.md): External command execution
- [Prompt Tools](./prompt.md): Prompt generation and management

## Tool Interface

All tools implement the `Tool` trait, which defines the interface for tool registration, schema definition, and execution:

```rust
#[async_trait]
pub trait Tool {
    fn get_name(&self) -> String;
    fn get_schema(&self) -> nixcode_llm_sdk::tools::Tool;
    async fn execute(
        &self,
        params: serde_json::Value,
        project: Arc<Project>,
    ) -> anyhow::Result<serde_json::Value>;
}
```

## Tool Registration

Tools are registered with the `Tools` struct, which maintains a collection of available tools:

```rust
pub struct Tools {
    pub(crate) hashmap: HashMap<String, SafeTool>,
}

impl Tools {
    pub fn new() -> Self {
        Self {
            hashmap: HashMap::new(),
        }
    }

    pub fn add_tool(&mut self, tool: SafeTool) {
        self.hashmap.insert(tool.get_name(), tool);
    }
}
```

Tools are registered in the `Nixcode::new` method:

```rust
tools.add_tool(Arc::new(SearchGlobFilesTool {}));
tools.add_tool(Arc::new(CreateFileTool {}));
tools.add_tool(Arc::new(ReadTextFileTool {}));
// ...
```

## Tool Configuration

Tools can be enabled or disabled through configuration:

```toml
[tools]
enabled = true

[tools.overrides]
git_add = false
read_text_file = true
```

The `Config` struct provides a method to check if a tool is enabled:

```rust
pub fn is_tool_enabled(&self, tool_name: &str) -> bool {
    // First check if we have a specific override for this tool
    if let Some(enabled) = self.tools.overrides.get(tool_name) {
        return *enabled;
    }

    // If not, use the global setting
    self.tools.enabled
}
```

## Tool Execution

Tools are executed by the `Nixcode` struct in response to LLM requests:

```rust
pub async fn execute_tools(self: &Arc<Self>) {
    let messages = self.messages.read().await;
    let message = messages.last();
    if message.is_none() {
        return;
    }
    let message = message.unwrap();

    if message.tool_calls.is_none() {
        return;
    }

    let tools = message.tool_calls.clone().unwrap();
    drop(messages);

    if tools.is_empty() {
        return;
    }

    let mut join_handles = vec![];
    self.messages.write().await.push(LLMMessage::user());

    for tool in tools {
        let handle = tokio::spawn({
            let nixcode = self.clone();
            async move {
                let (name, props) = tool.get_execute_params();

                if !nixcode.config.is_tool_enabled(name.as_str()) {
                    log::warn!("Tool {} is not enabled", name);
                    return;
                }

                let result = nixcode
                    .tools
                    .execute_tool(name.as_str(), props, nixcode.project.clone())
                    .await;

                let res = if let Ok(value) = result {
                    let value = serde_json::from_value(value).unwrap_or_else(|e| e.to_string());
                    tool.create_response(value)
                } else {
                    tool.create_response("Error executing tool".to_string())
                };

                let mut messages = nixcode.messages.write().await;
                let message = messages.last_mut().unwrap();
                message.add_tool_result(res);
                drop(messages);
            }
        });

        join_handles.push(handle);
    }

    join_all(join_handles).await;
    self.tx.send(NixcodeEvent::ToolsFinished).ok();
}
```

## Tool Macros

The library includes a procedural macro for simplifying tool definitions:

```rust
#[tool("Read file content")]
pub async fn read_text_file(
    params: ReadTextFileParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

This macro generates the boilerplate code for implementing the `Tool` trait.

## Adding New Tools

To add a new tool:

1. Create a new file in the appropriate category directory
2. Define the tool parameters using `schemars::JsonSchema`
3. Implement the tool function using the `#[tool]` macro
4. Register the tool in the `Nixcode::new` method

For example:

```rust
// tools/fs/example_tool.rs
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct ExampleToolParams {
    #[schemars(description = "Example parameter")]
    pub param: String,
}

#[tool("Example tool description")]
pub async fn example_tool(
    params: ExampleToolParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
    json!("Example result")
}

// In Nixcode::new
tools.add_tool(Arc::new(ExampleToolTool {}));
```