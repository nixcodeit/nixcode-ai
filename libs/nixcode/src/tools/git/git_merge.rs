use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitMergeParams {
    #[schemars(description = "Name of the branch to merge into the current branch")]
    pub branch_name: String,

    #[schemars(
        description = "Merge strategy: 'fast-forward' for fast-forward merge with commit, 'squash' for squash merge (default: fast-forward)"
    )]
    pub strategy: Option<String>,

    #[schemars(description = "Commit message for squash merge (required for squash strategy)")]
    pub commit_message: Option<String>,
}

#[tool("Merge a git branch into the current branch (fast-forward or squash only)")]
pub async fn git_merge(params: GitMergeParams, project: Arc<Project>) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());
    let branch_name = params.branch_name;
    let strategy = params
        .strategy
        .unwrap_or_else(|| "fast-forward".to_string());

    // Validate strategy
    if strategy != "fast-forward" && strategy != "squash" {
        return serde_json::json!(
            "Invalid merge strategy. Only 'fast-forward' and 'squash' are supported."
        );
    }

    // For squash merge, commit message is required
    if strategy == "squash" && params.commit_message.is_none() {
        return serde_json::json!("Commit message is required for squash merge strategy.");
    }

    // Check if branch exists
    let mut check_branch_cmd = Command::new("git");
    check_branch_cmd
        .current_dir(&current_dir)
        .arg("show-ref")
        .arg("--verify")
        .arg(format!("refs/heads/{}", branch_name));

    let branch_check = run_git_command(check_branch_cmd).await;
    if let Some(result) = branch_check.as_str() {
        if result.contains("error") || result.contains("fatal") {
            return serde_json::json!(format!("Branch '{}' does not exist", branch_name));
        }
    }

    // Execute merge based on strategy
    if strategy == "fast-forward" {
        // Fast-forward merge with commit
        let mut cmd = Command::new("git");
        cmd.current_dir(current_dir)
            .arg("merge")
            .arg("--no-ff") // Create a merge commit even if fast-forward is possible
            .arg(&branch_name);

        let output = run_git_command(cmd).await;

        // Check if the command was successful
        if let Some(result) = output.as_str() {
            if result.contains("error") || result.contains("fatal") || result.contains("conflict") {
                return serde_json::json!(format!("Merge failed: {}", result));
            }

            return serde_json::json!(format!(
                "Branch '{}' merged successfully with fast-forward strategy and commit",
                branch_name
            ));
        }

        return output;
    } else {
        // Squash merge
        let commit_message = params.commit_message.unwrap();

        // First do the squash merge without committing
        let mut squash_cmd = Command::new("git");
        squash_cmd
            .current_dir(&current_dir)
            .arg("merge")
            .arg("--squash")
            .arg(&branch_name);

        let squash_output = run_git_command(squash_cmd).await;

        // Check if the squash was successful
        if let Some(result) = squash_output.as_str() {
            if result.contains("error") || result.contains("fatal") || result.contains("conflict") {
                return serde_json::json!(format!("Squash merge failed: {}", result));
            }

            // Now commit the squashed changes
            let mut commit_cmd = Command::new("git");
            commit_cmd
                .current_dir(&current_dir)
                .arg("commit")
                .arg("-m")
                .arg(&commit_message);

            let commit_output = run_git_command(commit_cmd).await;

            // Check if the commit was successful
            if let Some(commit_result) = commit_output.as_str() {
                if commit_result.contains("error") || commit_result.contains("fatal") {
                    return serde_json::json!(format!(
                        "Squash merge succeeded but commit failed: {}",
                        commit_result
                    ));
                }

                return serde_json::json!(format!(
                    "Branch '{}' squash merged successfully",
                    branch_name
                ));
            }

            return commit_output;
        }

        return squash_output;
    }
}
