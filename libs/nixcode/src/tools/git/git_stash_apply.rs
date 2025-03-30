use std::sync::Arc;

use git2::StashApplyOptions;
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::utils::resolve_repository;
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
    let repository = resolve_repository(project.get_repo_path());
    if repository.is_none() {
        return json!("Not a git repository");
    }

    let mut repository = repository.unwrap();

    // Default stash index is 0 (most recent stash)
    let stash_index = props.stash_index.unwrap_or(0);
    let pop = props.pop.unwrap_or(false);

    // Apply the stash
    let mut apply_options = StashApplyOptions::new();

    let apply_result = match repository.stash_apply(stash_index, Some(&mut apply_options)) {
        Ok(_) => {
            // If pop is requested, also drop the stash
            if pop {
                match repository.stash_drop(stash_index) {
                    Ok(_) => json!(format!(
                        "Applied stash@{{{}}}, then dropped it",
                        stash_index
                    )),
                    Err(e) => json!(format!(
                        "Applied stash@{{{}}}, but failed to drop it: {}",
                        stash_index, e
                    )),
                }
            } else {
                json!(format!("Applied stash@{{{}}}", stash_index))
            }
        }
        Err(e) => json!(format!("Failed to apply stash@{{{}}}: {}", stash_index, e)),
    };

    apply_result
}
