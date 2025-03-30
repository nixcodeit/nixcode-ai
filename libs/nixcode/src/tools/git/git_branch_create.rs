use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::utils::resolve_repository;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitBranchCreateParams {
    #[schemars(description = "Name of the new branch to create")]
    pub branch_name: String,

    #[schemars(description = "Whether to switch to the newly created branch (default: false)")]
    pub switch: Option<bool>,
}

#[tool("Create a new git branch")]
pub async fn git_branch_create(
    params: GitBranchCreateParams,
    project: Arc<Project>,
) -> serde_json::Value {
    let repository = resolve_repository(project.get_repo_path());
    if repository.is_none() {
        return json!("Not a git repository");
    }

    let repository = repository.unwrap();
    let branch_name = params.branch_name;
    let switch = params.switch.unwrap_or(false);

    // Get HEAD commit to branch from
    let head = match repository.head() {
        Ok(head) => head,
        Err(e) => return json!(format!("Failed to get HEAD reference: {}", e)),
    };

    let commit = match head.peel_to_commit() {
        Ok(commit) => commit,
        Err(e) => return json!(format!("Failed to get HEAD commit: {}", e)),
    };

    // Create the branch
    let branch_result = repository.branch(&branch_name, &commit, false);

    match branch_result {
        Ok(branch) => {
            // Checkout the branch if requested
            if switch {
                let obj = branch.get().peel(git2::ObjectType::Any).unwrap();
                match repository.checkout_tree(&obj, None) {
                    Ok(_) => {
                        // Update HEAD to point to our branch now
                        match repository.set_head(&format!("refs/heads/{}", branch_name)) {
                            Ok(_) => json!(format!(
                                "Branch '{}' created and checked out successfully",
                                branch_name
                            )),
                            Err(e) => json!(format!(
                                "Branch '{}' created, but failed to update HEAD: {}",
                                branch_name, e
                            )),
                        }
                    }
                    Err(e) => json!(format!(
                        "Branch '{}' created, but failed to check it out: {}",
                        branch_name, e
                    )),
                }
            } else {
                json!(format!("Branch '{}' created successfully", branch_name))
            }
        }
        Err(e) => json!(format!("Failed to create branch '{}': {}", branch_name, e)),
    }
}
