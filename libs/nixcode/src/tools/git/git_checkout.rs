use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitCheckoutParams {
    /// Branch or commit to checkout
    pub target: String,

    /// Create a new branch with the given name
    #[serde(default)]
    pub create_branch: Option<bool>,
}

#[tool("Checkout git branch or commit")]
pub async fn git_checkout(params: GitCheckoutParams, project: Arc<Project>) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());

    let mut cmd = Command::new("git");
    cmd.current_dir(current_dir).arg("checkout");

    // Add -b flag if creating a new branch
    if params.create_branch.unwrap_or(false) {
        cmd.arg("-b");
    }

    // Add the target (branch name or commit hash)
    cmd.arg(&params.target);

    run_git_command(cmd).await
}
