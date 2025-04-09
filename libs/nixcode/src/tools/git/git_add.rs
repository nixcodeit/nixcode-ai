use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitAddParams {
    #[schemars(description = "Array of files that will be added to index")]
    pub files: Vec<String>,
}

#[tool("Track changes in git")]
pub async fn git_add(props: GitAddParams, project: Arc<Project>) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());

    let mut cmd = Command::new("git");
    cmd.current_dir(current_dir).arg("add");

    // Add each file to the command
    for file in &props.files {
        cmd.arg(file);
    }

    let output = run_git_command(cmd).await;

    // If the command was successful but had no output, generate a success message
    if let Some(result) = output.as_str() {
        if result.trim().is_empty() {
            let mut result = String::new();
            for file in props.files {
                result.push_str(&format!("Added {}\n", file));
            }
            return serde_json::json!(result.trim());
        }
    }

    output
}
