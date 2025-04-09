use std::path::Path;
use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitDiffProps {
    #[schemars(description = "Path to the file to show diff for")]
    pub file_path: String,
}

#[tool("Get file diff")]
pub async fn git_diff(props: GitDiffProps, project: Arc<Project>) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());

    // Check if file exists
    let full_path = current_dir.join(&props.file_path);
    if !Path::new(&full_path).exists() {
        return serde_json::json!(format!("File not found: {}", props.file_path));
    }

    // First try to get diff for the file (including staged changes)
    let mut cmd = Command::new("git");
    cmd.current_dir(&current_dir)
        .arg("diff")
        .arg("HEAD")
        .arg("--")
        .arg(&props.file_path);

    let output = run_git_command(cmd).await;

    // If there's no output, the file might be new or only staged
    if let Some(result) = output.as_str() {
        if result.trim().is_empty() {
            // Check if the file is staged
            let mut staged_cmd = Command::new("git");
            staged_cmd
                .current_dir(&current_dir)
                .arg("diff")
                .arg("--cached")
                .arg("--")
                .arg(&props.file_path);

            let staged_output = run_git_command(staged_cmd).await;

            if let Some(staged_result) = staged_output.as_str() {
                if !staged_result.trim().is_empty() {
                    return staged_output;
                }
            }

            // If the file is new and not tracked, show it as a new file
            let mut status_cmd = Command::new("git");
            status_cmd
                .current_dir(&current_dir)
                .arg("status")
                .arg("--porcelain")
                .arg("--")
                .arg(&props.file_path);

            let status_output = run_git_command(status_cmd).await;

            if let Some(status_result) = status_output.as_str() {
                if status_result.trim().starts_with("??") {
                    // It's a new untracked file, read its contents
                    let mut cat_cmd = Command::new("cat");
                    cat_cmd.current_dir(&current_dir).arg(&props.file_path);

                    let cat_output = run_git_command(cat_cmd).await;

                    if let Some(content) = cat_output.as_str() {
                        return serde_json::json!(format!(
                            "New file: {}\n\n{}",
                            props.file_path, content
                        ));
                    }
                }
            }

            return serde_json::json!(format!("No changes detected for file: {}", props.file_path));
        }

        return output;
    }

    output
}
