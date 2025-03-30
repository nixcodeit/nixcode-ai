use std::path::PathBuf;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct CreateFileParams {
    #[schemars(description = "Relative path to new file")]
    pub path: String,
}

#[tool("Create empty file in given path")]
pub async fn create_file(
    params: CreateFileParams,
    project: std::sync::Arc<Project>,
) -> serde_json::Value {
    use crate::utils::fs;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    let file_path = PathBuf::from(params.path);

    let cwd = project.get_cwd();
    let path = fs::join_path(cwd.clone(), file_path);
    if path.is_err() {
        return json!(path.unwrap_err().to_string());
    }

    let path = path.unwrap();
    if !path.starts_with(cwd) {
        return json!("Path must be inside project directory");
    }

    // create directories if they don't exist
    let parent = path.parent().unwrap();
    let create_dirs_result = tokio::fs::create_dir_all(parent).await;
    if create_dirs_result.is_err() {
        return json!(create_dirs_result.unwrap_err().to_string());
    }

    let file = File::create(&path).await;

    match file {
        Ok(mut f) => {
            f.write_all(b"").await.unwrap();
            json!("File created")
        }
        Err(e) => json!(e.to_string()),
    }
}
