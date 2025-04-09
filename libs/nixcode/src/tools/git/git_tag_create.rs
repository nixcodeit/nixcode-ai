use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitTagCreateParams {
    #[schemars(description = "Name of the tag to create")]
    pub tag_name: String,

    #[schemars(description = "Optional message for annotated tag")]
    pub message: Option<String>,

    #[schemars(description = "Commit hash to tag (default: HEAD)")]
    pub target: Option<String>,

    #[schemars(description = "Force overwrite of existing tag (default: false)")]
    pub force: Option<bool>,
}

#[tool("Create a git tag")]
pub async fn git_tag_create(
    params: GitTagCreateParams,
    project: Arc<Project>,
) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());
    let tag_name = params.tag_name;
    let message = params.message;
    let target = params.target;
    let force = params.force.unwrap_or(false);

    let mut cmd = Command::new("git");
    cmd.current_dir(current_dir).arg("tag");

    // Add force flag if requested
    if force {
        cmd.arg("-f");
    }

    // Add message if provided (creates annotated tag)
    if let Some(msg) = &message {
        cmd.arg("-a").arg("-m").arg(msg);
    }

    // Add the tag name
    cmd.arg(&tag_name);

    // Add target commit if provided
    if let Some(target_commit) = &target {
        cmd.arg(target_commit);
    }

    let output = run_git_command(cmd).await;

    // Check if the command was successful
    if let Some(result) = output.as_str() {
        if result.contains("error") || result.contains("fatal") {
            return output;
        }

        if message.is_some() {
            return serde_json::json!(format!("Annotated tag '{}' created successfully", tag_name));
        } else {
            return serde_json::json!(format!(
                "Lightweight tag '{}' created successfully",
                tag_name
            ));
        }
    }

    output
}
