use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitStashListParams {
    // Empty parameters as this command doesn't require any
}

#[tool("List all stashes in git repository")]
pub async fn git_stash_list(
    _props: GitStashListParams,
    project: Arc<Project>,
) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());
    
    let mut cmd = Command::new("git");
    cmd.current_dir(current_dir)
       .arg("stash")
       .arg("list");
    
    let output = run_git_command(cmd).await;
    
    // Check if the command was successful
    if let Some(result) = output.as_str() {
        if result.trim().is_empty() {
            return serde_json::json!("No stashes found");
        }
        
        return output;
    }
    
    output
}