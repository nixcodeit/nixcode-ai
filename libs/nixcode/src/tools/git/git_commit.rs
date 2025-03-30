use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::utils::resolve_repository;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitCommitProps {
    #[schemars(description = "Message for commit")]
    pub message: String,
}

#[tool("Commit changes")]
pub async fn git_commit(props: GitCommitProps, project: Arc<Project>) -> serde_json::Value {
    let repository = resolve_repository(project.get_repo_path());
    if repository.is_none() {
        return json!("Not a git repository");
    }

    let repo = repository.unwrap();

    let index = repo.index();
    if let Err(e) = index {
        return json!(format!("Cannot get index, reason: {}", e));
    }
    let mut index = index.unwrap();
    let oid = index.write_tree();
    if let Err(e) = oid {
        return json!(format!("Cannot write index tree, reason: {}", e));
    }
    let oid = oid.unwrap();

    let signature = repo.signature();
    if let Err(e) = signature {
        return json!(format!("Cannot get signature for commiter, reason: {}", e));
    }
    let signature = signature.unwrap();
    let tree = repo.find_tree(oid);
    if let Err(e) = tree {
        return json!(format!("Cannot find tree for oid: {}, reason: {}", oid, e));
    }
    let tree = tree.unwrap();

    let parent_commit = repo.head();
    if let Err(e) = parent_commit {
        return json!(format!("Cannot get HEAD ref, reason: {}", e));
    }

    let parent_commit = parent_commit.unwrap().peel_to_commit();
    if let Err(e) = parent_commit {
        return json!(format!("Cannot peel to commit, reason: {}", e));
    }
    let parent_commit = parent_commit.unwrap();

    let result = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        props.message.as_str(),
        &tree,
        &[&parent_commit],
    );

    if let Err(e) = result {
        return json!(format!("Can't commit, reason: {}", e));
    }

    json!("Commit created")
}
