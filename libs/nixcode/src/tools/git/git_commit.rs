use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitCommitProps {
    #[schemars(description = "Message for commit")]
    pub message: String,
}

#[tool("Commit changes")]
pub async fn git_commit(props: GitCommitProps, project: Arc<Project>) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());
    
    let mut cmd = Command::new("git");
    cmd.current_dir(current_dir)
       .arg("commit")
       .arg("-m")
       .arg(&props.message);
    
    let output = run_git_command(cmd).await;
    
    // If the command was successful but had no clear output, provide a standard message
    if let Some(result) = output.as_str() {
        if result.contains("nothing to commit") {
            return serde_json::json!("Nothing to commit, working tree clean");
        } else if result.contains("[") && result.contains("]") {
            // This indicates a successful commit with branch and commit hash info
            return serde_json::json!("Commit created");
        }
    }
    
    output
}