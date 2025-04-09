use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitLogProps {
    #[schemars(description = "Starting reference (commit hash, branch name, or tag)")]
    pub from_ref: Option<String>,

    #[schemars(description = "Ending reference (commit hash, branch name, or tag)")]
    pub to_ref: Option<String>,

    #[schemars(description = "Maximum number of commits to retrieve")]
    pub limit: Option<usize>,

    #[schemars(description = "Path to limit the history to a specific file or directory")]
    pub path: Option<String>,
}

#[tool("Get git log between refs")]
pub async fn git_log(props: GitLogProps, project: Arc<Project>) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());
    let limit = props.limit.unwrap_or(50); // Default to 50 commits

    let mut cmd = Command::new("git");
    cmd.current_dir(current_dir)
        .arg("log")
        .arg("--pretty=format:(%h) by: %an <%ae> [%ad]%n%s%n%b")
        .arg("--date=format:%Y-%m-%d %H:%M:%S")
        .arg("-n")
        .arg(limit.to_string());

    // Add range if specified
    if props.from_ref.is_some() || props.to_ref.is_some() {
        let range = match (&props.from_ref, &props.to_ref) {
            (Some(from), Some(to)) => format!("{}..{}", from, to),
            (Some(from), None) => format!("{}..HEAD", from),
            (None, Some(to)) => format!("{}", to),
            (None, None) => "HEAD".to_string(),
        };
        cmd.arg(range);
    }

    // Add path if specified
    if let Some(path) = &props.path {
        cmd.arg("--").arg(path);
    }

    let output = run_git_command(cmd).await;

    // Format the output
    if let Some(result) = output.as_str() {
        if result.trim().is_empty() {
            return serde_json::json!("No commits found");
        }

        // The git log command already formats the output as we want
        return output;
    }

    output
}
