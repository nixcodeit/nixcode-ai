use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize, Default)]
pub struct GitBranchesProps {
    #[schemars(description = "Show all branches including remotes (default: false)")]
    pub all: Option<bool>,
}

#[tool("Display git branches")]
pub async fn git_branches(props: GitBranchesProps, project: Arc<Project>) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());
    let show_all = props.all.unwrap_or(false);

    let mut cmd = Command::new("git");
    cmd.current_dir(current_dir).arg("branch");

    if show_all {
        cmd.arg("--all");
    }

    // Add color=always to preserve the formatting
    cmd.arg("--color=never");

    let output = run_git_command(cmd).await;

    // Format the output
    if let Some(result) = output.as_str() {
        if result.trim().is_empty() {
            return serde_json::json!("No branches found");
        }

        let formatted = format!("Branches:\n{}", result);
        return serde_json::json!(formatted);
    }

    output
}
