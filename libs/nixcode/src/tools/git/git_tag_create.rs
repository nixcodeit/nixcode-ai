use std::sync::Arc;

use git2::Oid;
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::utils::resolve_repository;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitTagCreateParams {
    #[schemars(description = "Name of the tag to create")]
    pub tag_name: String,

    #[schemars(description = "Optional message for annotated tag")]
    pub message: Option<String>,

    #[schemars(description = "Commit hash to tag (default: HEAD)")]
    pub target: Option<String>,

    #[schemars(description = "Force overwrite of existing tag (default: false)")]
    pub force: Option<bool>,
}

#[tool("Create a git tag")]
pub async fn git_tag_create(
    params: GitTagCreateParams,
    project: Arc<Project>,
) -> serde_json::Value {
    let repository = resolve_repository(project.get_repo_path());
    if repository.is_none() {
        return json!("Not a git repository");
    }

    let repository = repository.unwrap();
    let tag_name = params.tag_name;
    let message = params.message;
    let target_str = params.target;
    let force = params.force.unwrap_or(false);

    // Get the target commit
    let target_oid = match target_str {
        Some(target) => {
            // Parse the provided commit hash
            match Oid::from_str(&target) {
                Ok(oid) => oid,
                Err(e) => return json!(format!("Invalid commit hash '{}': {}", target, e)),
            }
        }
        None => {
            // Use HEAD
            match repository.head() {
                Ok(head) => match head.target() {
                    Some(oid) => oid,
                    None => return json!("HEAD reference is invalid"),
                },
                Err(e) => return json!(format!("Failed to get HEAD: {}", e)),
            }
        }
    };

    // Find the target commit
    let target_commit = match repository.find_commit(target_oid) {
        Ok(commit) => commit,
        Err(e) => return json!(format!("Failed to find commit: {}", e)),
    };

    let tag_oid = Oid::from_str(tag_name.as_str());

    if let Err(e) = tag_oid {
        return json!(format!("Invalid tag name '{}': {}", tag_name, e));
    }

    let tag_oid = tag_oid.unwrap();

    // Check if the tag already exists
    if repository.find_tag(tag_oid).is_ok() && !force {
        return json!(format!(
            "Tag '{}' already exists. Use force option to override.",
            tag_name
        ));
    }

    let tagger = match repository.signature() {
        Ok(sig) => sig,
        Err(e) => return json!(format!("Failed to get signature: {}", e)),
    };

    // Create the tag
    let result = match message {
        // Create an annotated tag with message
        Some(msg) => {
            let tag_oid = repository.tag(
                &tag_name,
                &target_commit.into_object(),
                &tagger,
                &msg,
                force,
            );
            match tag_oid {
                Ok(_) => json!(format!("Annotated tag '{}' created successfully", tag_name)),
                Err(e) => json!(format!(
                    "Failed to create annotated tag '{}': {}",
                    tag_name, e
                )),
            }
        }
        // Create a lightweight tag
        None => {
            let result = repository.tag_lightweight(&tag_name, &target_commit.into_object(), force);
            match result {
                Ok(_) => json!(format!(
                    "Lightweight tag '{}' created successfully",
                    tag_name
                )),
                Err(e) => json!(format!(
                    "Failed to create lightweight tag '{}': {}",
                    tag_name, e
                )),
            }
        }
    };

    result
}
