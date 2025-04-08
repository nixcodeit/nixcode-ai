# Tool Trait

The `Tool` trait defines the interface that all tools must implement to be usable by the nixcode library.

## Overview

The `Tool` trait is defined in `tools/mod.rs` and provides a standardized interface for tool registration, schema definition, and execution. All tools in the nixcode library implement this trait.

## Definition

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

### Methods

#### `get_name`

```rust
fn get_name(&self) -> String
```

Returns the name of the tool. This name is used to identify the tool in the LLM's function calling interface.

#### `get_schema`

```rust
fn get_schema(&self) -> nixcode_llm_sdk::tools::Tool
```

Returns the JSON schema for the tool. This schema is used by the LLM to understand the tool's parameters and functionality.

#### `execute`

```rust
async fn execute(
    &self,
    params: serde_json::Value,
    project: Arc<Project>,
) -> anyhow::Result<serde_json::Value>
```

Executes the tool with the provided parameters and project context. This method is called when the LLM requests the tool's execution.

## Tool Macro

The nixcode library includes a procedural macro for simplifying tool definitions:

```rust
#[tool("Read file content")]
pub async fn read_text_file(
    params: ReadTextFileParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Implementation
}
```

This macro generates the boilerplate code for implementing the `Tool` trait, including the `get_name`, `get_schema`, and `execute` methods.

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

## Example Implementation

Here's an example of a simple tool implementation:

```rust
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
```

This tool can then be registered with the `Tools` struct:

```rust
tools.add_tool(Arc::new(ExampleToolTool {}));
```

## Tool Configuration

Tools can be enabled or disabled through configuration:

```toml
[tools]
enabled = true

[tools.overrides]
example_tool = false
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