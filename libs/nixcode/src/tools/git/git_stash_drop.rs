use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::utils::resolve_repository;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitStashDropParams {
    #[schemars(description = "Stash index to drop (0 is the most recent stash)")]
    pub stash_index: Option<usize>,
}

#[tool("Drop a stash from git stash list")]
pub async fn git_stash_drop(props: GitStashDropParams, project: Arc<Project>) -> serde_json::Value {
    let repository = resolve_repository(project.get_repo_path());
    if repository.is_none() {
        return json!("Not a git repository");
    }

    let mut repository = repository.unwrap();

    // Default stash index is 0 (most recent stash)
    let stash_index = props.stash_index.unwrap_or(0);

    // Get the stash message before dropping (for better feedback)
    let mut stash_message = String::new();
    let _ = repository
        .stash_foreach(|i, message, _| {
            if i == stash_index {
                stash_message = message.to_string();
                false
            } else {
                true
            }
        })
        .is_ok();

    // Drop the stash
    match repository.stash_drop(stash_index) {
        Ok(_) => {
            if !stash_message.is_empty() {
                json!(format!(
                    "Dropped stash@{{{}}}: {}",
                    stash_index, stash_message
                ))
            } else {
                json!(format!("Dropped stash@{{{}}}", stash_index))
            }
        }
        Err(e) => {
            if e.code() == git2::ErrorCode::NotFound {
                json!(format!("No stash found at index {}", stash_index))
            } else {
                json!(format!("Failed to drop stash@{{{}}}: {}", stash_index, e))
            }
        }
    }
}
