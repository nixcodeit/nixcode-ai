use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitGetTreeProps {}

#[tool("Get git status")]
pub async fn git_status(_: GitGetTreeProps, project: Arc<Project>) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());

    let mut cmd = Command::new("git");
    cmd.current_dir(current_dir)
        .arg("status")
        .arg("--porcelain");  // Removed -z flag to get newline-separated output

    let output = run_git_command(cmd).await;

    // If the output is empty or just whitespace, the working tree is clean
    if let Some(result) = output.as_str() {
        if result.trim().is_empty() {
            return serde_json::json!("Working tree clean");
        }

        // The output is already formatted correctly with --porcelain
        // Just return it directly
        return serde_json::json!(result);
    }

    // If we get here, there was likely an error
    output
}