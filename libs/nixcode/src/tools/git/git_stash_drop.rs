use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitStashDropParams {
    #[schemars(description = "Stash index to drop (0 is the most recent stash)")]
    pub stash_index: Option<usize>,
}

#[tool("Drop a stash from git stash list")]
pub async fn git_stash_drop(props: GitStashDropParams, project: Arc<Project>) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());

    // Default stash index is 0 (most recent stash)
    let stash_index = props.stash_index.unwrap_or(0);

    // First get the stash info for better feedback
    let mut stash_info_cmd = Command::new("git");
    stash_info_cmd
        .current_dir(&current_dir)
        .arg("stash")
        .arg("list")
        .arg("-n")
        .arg("1")
        .arg(format!("--skip={}", stash_index));

    let stash_info = run_git_command(stash_info_cmd).await;
    let stash_message = if let Some(info) = stash_info.as_str() {
        info.trim().to_string()
    } else {
        String::new()
    };

    // Now drop the stash
    let mut cmd = Command::new("git");
    cmd.current_dir(current_dir).arg("stash").arg("drop");

    // Add stash reference if not the default
    if stash_index > 0 {
        cmd.arg(format!("stash@{{{}}}", stash_index));
    }

    let output = run_git_command(cmd).await;

    // Check if the command was successful
    if let Some(result) = output.as_str() {
        if result.contains("error") || result.contains("fatal") {
            if result.contains("No stash found") {
                return serde_json::json!(format!("No stash found at index {}", stash_index));
            }
            return output;
        }

        if !stash_message.is_empty() {
            return serde_json::json!(format!(
                "Dropped stash@{{{}}}: {}",
                stash_index, stash_message
            ));
        } else {
            return serde_json::json!(format!("Dropped stash@{{{}}}", stash_index));
        }
    }

    output
}
