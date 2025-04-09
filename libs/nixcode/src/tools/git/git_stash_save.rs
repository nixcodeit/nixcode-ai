use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitStashSaveParams {
    #[schemars(description = "Optional message describing the stashed changes")]
    pub message: Option<String>,
}

#[tool("Save changes in git stash")]
pub async fn git_stash_save(props: GitStashSaveParams, project: Arc<Project>) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());

    let mut cmd = Command::new("git");
    cmd.current_dir(current_dir).arg("stash");

    // Add message if provided
    if let Some(message) = &props.message {
        cmd.arg("push").arg("-m").arg(message);
    } else {
        cmd.arg("push");
    }

    let output = run_git_command(cmd).await;

    // Check if the command was successful
    if let Some(result) = output.as_str() {
        if result.contains("No local changes to save") {
            return serde_json::json!("No local changes to save");
        } else if result.contains("Saved working directory") {
            let message_text = props.message.unwrap_or_else(|| "WIP on stash".to_string());
            return serde_json::json!(format!(
                "Saved working directory and index state: {}",
                message_text
            ));
        }
    }

    output
}
