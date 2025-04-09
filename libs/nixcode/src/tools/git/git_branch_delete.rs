use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitBranchDeleteParams {
    #[schemars(description = "Name of the branch to delete")]
    pub branch_name: String,

    #[schemars(description = "Force deletion even if branch is not fully merged (default: false)")]
    pub force: Option<bool>,
}

#[tool("Delete a git branch")]
pub async fn git_branch_delete(
    params: GitBranchDeleteParams,
    project: Arc<Project>,
) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());
    let branch_name = params.branch_name;
    let force = params.force.unwrap_or(false);

    // First check if we're on the branch we're trying to delete
    let mut current_branch_cmd = Command::new("git");
    current_branch_cmd
        .current_dir(&current_dir)
        .arg("symbolic-ref")
        .arg("--short")
        .arg("HEAD");

    let current_branch_output = run_git_command(current_branch_cmd).await;

    if let Some(current_branch) = current_branch_output.as_str() {
        if current_branch.trim() == branch_name {
            return serde_json::json!(format!(
                "Cannot delete the currently checked out branch '{}'",
                branch_name
            ));
        }
    }

    // Now delete the branch
    let mut cmd = Command::new("git");
    cmd.current_dir(current_dir).arg("branch");

    if force {
        cmd.arg("-D"); // Force delete
    } else {
        cmd.arg("-d"); // Normal delete
    }

    cmd.arg(&branch_name);

    let output = run_git_command(cmd).await;

    // Check if the command was successful
    if let Some(result) = output.as_str() {
        if result.contains("error") || result.contains("fatal") {
            return output;
        }

        return serde_json::json!(format!("Branch '{}' deleted successfully", branch_name));
    }

    output
}
