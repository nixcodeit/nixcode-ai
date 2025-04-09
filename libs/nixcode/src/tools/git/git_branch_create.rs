use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitBranchCreateParams {
    #[schemars(description = "Name of the new branch to create")]
    pub branch_name: String,

    #[schemars(description = "Whether to switch to the newly created branch (default: false)")]
    pub switch: Option<bool>,
}

#[tool("Create a new git branch")]
pub async fn git_branch_create(
    params: GitBranchCreateParams,
    project: Arc<Project>,
) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());
    let branch_name = params.branch_name;
    let switch = params.switch.unwrap_or(false);

    let mut cmd = Command::new("git");

    if switch {
        // Use checkout -b to create and switch to the branch
        cmd.current_dir(current_dir)
            .arg("checkout")
            .arg("-b")
            .arg(&branch_name);
    } else {
        // Just create the branch without switching
        cmd.current_dir(current_dir).arg("branch").arg(&branch_name);
    }

    let output = run_git_command(cmd).await;

    // Check if the command was successful
    if let Some(result) = output.as_str() {
        if result.contains("error") || result.contains("fatal") {
            return output;
        }

        if switch {
            return serde_json::json!(format!(
                "Branch '{}' created and checked out successfully",
                branch_name
            ));
        } else {
            return serde_json::json!(format!("Branch '{}' created successfully", branch_name));
        }
    }

    output
}
