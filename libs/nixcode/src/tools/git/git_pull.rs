use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitPullParams {
    /// Remote name (default: origin)
    #[serde(default)]
    pub remote: Option<String>,

    /// Branch name (default: current branch)
    #[serde(default)]
    pub branch: Option<String>,
}

#[tool("Pull changes from remote git repository")]
pub async fn git_pull(params: GitPullParams, project: Arc<Project>) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());

    let mut cmd = Command::new("git");
    cmd.current_dir(current_dir).arg("pull");

    // Add remote name if provided
    if let Some(remote) = params.remote {
        cmd.arg(remote);

        // Add branch name if provided (only if remote is also provided)
        if let Some(branch) = params.branch {
            cmd.arg(branch);
        }
    }

    run_git_command(cmd).await
}
