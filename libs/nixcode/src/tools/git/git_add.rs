use std::path::PathBuf;
use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::utils::resolve_repository;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitAddParams {
    #[schemars(description = "Array of files that will be added to index")]
    pub files: Vec<String>,
}

#[tool("Track changes in git")]
pub async fn git_add(props: GitAddParams, project: Arc<Project>) -> serde_json::Value {
    let repository = resolve_repository(project.get_repo_path());
    if repository.is_none() {
        return json!("Not a git repository");
    }

    let repository = repository.unwrap();

    let index = repository.index();

    if let Err(e) = index {
        return json!(e.to_string());
    }

    let mut index = index.unwrap();
    let mut result = String::new();

    for file_path in props.files {
        let path = PathBuf::from(file_path.clone());
        if let Err(e) = index.add_path(path.as_path()) {
            result.push_str(format!("Cannot add {}, reason: {}\n", file_path, e).as_str());
        } else {
            result.push_str(format!("Added {}\n", file_path).as_str());
        }
    }

    let write_result = index.write();
    if let Err(e) = write_result {
        return json!(format!("Cannot save index, reason: {}", e));
    }

    serde_json::to_value(result).unwrap()
}
