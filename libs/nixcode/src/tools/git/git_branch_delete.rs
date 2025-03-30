use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::utils::resolve_repository;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitBranchDeleteParams {
    #[schemars(description = "Name of the branch to delete")]
    pub branch_name: String,

    #[schemars(description = "Force deletion even if branch is not fully merged (default: false)")]
    pub force: Option<bool>,
}

#[tool("Delete a git branch")]
pub async fn git_branch_delete(
    params: GitBranchDeleteParams,
    project: Arc<Project>,
) -> serde_json::Value {
    let repository = resolve_repository(project.get_repo_path());
    if repository.is_none() {
        return json!("Not a git repository");
    }

    let repository = repository.unwrap();
    let branch_name = params.branch_name;
    let force = params.force.unwrap_or(false);

    // Get current branch to make sure we're not deleting it
    let head = match repository.head() {
        Ok(head) => head,
        Err(e) => return json!(format!("Failed to get HEAD reference: {}", e)),
    };

    let current_branch_name = match head.shorthand() {
        Some(name) => name.to_string(),
        None => String::from("(detached HEAD)"),
    };

    if current_branch_name == branch_name {
        return json!(format!(
            "Cannot delete the currently checked out branch '{}'",
            branch_name
        ));
    }

    // Find the branch
    let branch_result = repository.find_branch(&branch_name, git2::BranchType::Local);

    match branch_result {
        Ok(mut branch) => {
            // Check if branch is fully merged if not forcing deletion
            if !force && branch.is_head() {
                return json!(format!(
                    "Cannot delete branch '{}' as it is the current HEAD",
                    branch_name
                ));
            }

            // Delete the branch
            match branch.delete() {
                Ok(_) => json!(format!("Branch '{}' deleted successfully", branch_name)),
                Err(e) => json!(format!("Failed to delete branch '{}': {}", branch_name, e)),
            }
        }
        Err(e) => json!(format!("Failed to find branch '{}': {}", branch_name, e)),
    }
}
