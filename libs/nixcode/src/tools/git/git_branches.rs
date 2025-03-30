use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::utils::resolve_repository;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize, Default)]
pub struct GitBranchesProps {
    #[schemars(description = "Show all branches including remotes (default: false)")]
    pub all: Option<bool>,
}

#[tool("Display git branches")]
pub async fn git_branches(props: GitBranchesProps, project: Arc<Project>) -> serde_json::Value {
    let repository = resolve_repository(project.get_repo_path());
    if repository.is_none() {
        return json!("Not a git repository");
    }

    let repository = repository.unwrap();
    let show_all = props.all.unwrap_or(false);

    // Get current branch name
    let head = match repository.head() {
        Ok(head) => head,
        Err(e) => return json!(format!("Failed to get HEAD: {}", e)),
    };

    let current_branch_name = match head.shorthand() {
        Some(name) => name.to_string(),
        None => String::from("(detached HEAD)"),
    };

    // Get all branches
    let branches = repository.branches(None);
    if let Err(e) = branches {
        return json!(format!("Failed to get branches: {}", e));
    }

    let mut result = String::new();
    result.push_str("Branches:\n");

    // Format branches
    for branch_result in branches.unwrap() {
        if let Ok((branch, branch_type)) = branch_result {
            let branch_name = match branch.name() {
                Ok(Some(name)) => name.to_string(),
                _ => continue,
            };

            // Skip remote branches if not showing all
            if !show_all && branch_type == git2::BranchType::Remote {
                continue;
            }

            let prefix = if branch_type == git2::BranchType::Remote {
                "remote: "
            } else {
                ""
            };

            // Mark current branch with *
            let is_current =
                branch_type == git2::BranchType::Local && branch_name == current_branch_name;
            let marker = if is_current { "* " } else { "  " };

            result.push_str(&format!("{}{}{}\n", marker, prefix, branch_name));
        }
    }

    json!(result)
}
