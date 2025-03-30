use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::utils::resolve_repository;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitStashListParams {
    // Empty parameters as this command doesn't require any
}

#[derive(Serialize)]
struct StashEntry {
    index: usize,
    message: String,
}

#[tool("List all stashes in git repository")]
pub async fn git_stash_list(
    _props: GitStashListParams,
    project: Arc<Project>,
) -> serde_json::Value {
    let repository = resolve_repository(project.get_repo_path());
    if repository.is_none() {
        return json!("Not a git repository");
    }

    let mut repository = repository.unwrap();

    // Initialize an empty result vector
    let mut stash_list = String::new();

    // Callback to process each stash entry
    let mut callback = |index: usize, message: &str, _stash_id: &git2::Oid| -> bool {
        let line = format!("stash@{{{}}}: {}\n", index, message);
        stash_list.push_str(&line);
        true // continue with next stash
    };

    // Iterate through all stashes
    match repository.stash_foreach(&mut callback) {
        Ok(_) => {
            if stash_list.is_empty() {
                json!("No stashes found")
            } else {
                json!(stash_list)
            }
        }
        Err(e) => json!(format!("Failed to list stashes: {}", e)),
    }
}
