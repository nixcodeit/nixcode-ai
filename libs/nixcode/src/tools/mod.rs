use crate::project::Project;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub mod fs;
pub mod git;
pub mod glob;
pub mod prompt;
pub mod search;

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

pub type SafeTool = Arc<dyn Tool + Send + Sync>;

#[derive(Default, Clone)]
pub struct Tools {
    hashmap: HashMap<String, SafeTool>,
}

impl Tools {
    pub fn new() -> Self {
        Self {
            hashmap: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.hashmap.len() == 0
    }

    pub fn add_tool(&mut self, tool: SafeTool) {
        self.hashmap.insert(tool.get_name(), tool);
    }

    pub fn get_tool(&self, name: &str) -> Option<SafeTool> {
        self.hashmap.get(name).cloned()
    }

    pub fn get_all_tools(&self) -> Vec<nixcode_llm_sdk::tools::Tool> {
        self.hashmap
            .values()
            .map(|tool| tool.get_schema())
            .collect()
    }

    pub async fn execute_tool(
        &self,
        name: &str,
        params: serde_json::Value,
        project: Arc<Project>,
    ) -> anyhow::Result<serde_json::Value> {
        if let Some(tool) = self.get_tool(name) {
            tool.execute(params, project).await
        } else {
            Err(anyhow::anyhow!("Tool not found"))
        }
    }
}
