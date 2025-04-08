use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitStashApplyParams {
    #[schemars(description = "Stash index to apply (0 is the most recent stash)")]
    pub stash_index: Option<usize>,

    #[schemars(description = "Whether to drop the stash after applying it")]
    pub pop: Option<bool>,
}

#[tool("Apply changes from git stash")]
pub async fn git_stash_apply(
    props: GitStashApplyParams,
    project: Arc<Project>,
) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());
    
    // Default stash index is 0 (most recent stash)
    let stash_index = props.stash_index.unwrap_or(0);
    let pop = props.pop.unwrap_or(false);
    
    let mut cmd = Command::new("git");
    cmd.current_dir(current_dir);
    
    if pop {
        cmd.arg("stash")
           .arg("pop");
    } else {
        cmd.arg("stash")
           .arg("apply");
    }
    
    // Add stash reference if not the default
    if stash_index > 0 {
        cmd.arg(format!("stash@{{{}}}", stash_index));
    }
    
    let output = run_git_command(cmd).await;
    
    // Check if the command was successful
    if let Some(result) = output.as_str() {
        if result.contains("error") || result.contains("fatal") {
            return output;
        }
        
        if pop {
            return serde_json::json!(format!("Applied stash@{{{}}}, then dropped it", stash_index));
        } else {
            return serde_json::json!(format!("Applied stash@{{{}}}", stash_index));
        }
    }
    
    output
}