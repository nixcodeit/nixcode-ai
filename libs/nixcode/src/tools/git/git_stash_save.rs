use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::utils::resolve_repository;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitStashSaveParams {
    #[schemars(description = "Optional message describing the stashed changes")]
    pub message: Option<String>,
}

#[tool("Save changes in git stash")]
pub async fn git_stash_save(props: GitStashSaveParams, project: Arc<Project>) -> serde_json::Value {
    let repository = resolve_repository(project.get_repo_path());
    if repository.is_none() {
        return json!("Not a git repository");
    }

    let mut repository = repository.unwrap();

    // Get signature for the stash
    let signature = match repository.signature() {
        Ok(sig) => sig,
        Err(e) => return json!(format!("Failed to get signature: {}", e)),
    };

    // Setup default message
    let message = props.message.unwrap_or_else(|| "WIP on stash".to_string());

    // Try to create a stash
    match repository.stash_save(&signature, &message, None) {
        Ok(stash_id) => {
            if stash_id.is_zero() {
                json!("No local changes to save")
            } else {
                json!(format!(
                    "Saved working directory and index state: {}",
                    message
                ))
            }
        }
        Err(e) => json!(format!("Failed to stash changes: {}", e)),
    }
}
