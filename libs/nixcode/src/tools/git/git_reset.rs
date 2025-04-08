use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitResetParams {
    /// Target commit or reference (default: HEAD)
    #[serde(default)]
    pub target: Option<String>,
    
    /// Reset mode: soft, mixed, or hard (default: mixed)
    #[serde(default)]
    pub mode: Option<String>,
    
    /// Specific file paths to reset (default: all files)
    #[serde(default)]
    pub paths: Option<Vec<String>>,
}

#[tool("Reset git working tree or index")]
pub async fn git_reset(params: GitResetParams, project: Arc<Project>) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());
    
    let mut cmd = Command::new("git");
    cmd.current_dir(current_dir)
       .arg("reset");
    
    // Add mode flag if provided
    if let Some(mode) = params.mode {
        match mode.to_lowercase().as_str() {
            "soft" => { cmd.arg("--soft"); },
            "hard" => { cmd.arg("--hard"); },
            "mixed" => { cmd.arg("--mixed"); },
            _ => {} // Default is mixed, so no flag needed
        }
    }
    
    // Add target if provided
    if let Some(target) = params.target {
        cmd.arg(target);
    }
    
    // Add specific file paths if provided
    if let Some(paths) = params.paths {
        // Add -- separator before paths
        if !paths.is_empty() {
            cmd.arg("--");
            for path in paths {
                cmd.arg(path);
            }
        }
    }
    
    run_git_command(cmd).await
}