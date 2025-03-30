use std::path::PathBuf;
use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct DeleteFileParams {
    #[schemars(description = "Relative path to file")]
    pub path: String,
}

#[tool("Delete file")]
pub async fn delete_file(params: DeleteFileParams, project: Arc<Project>) -> serde_json::Value {
    use crate::utils::fs;
    use tokio::fs::remove_file;

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

    let file = remove_file(&path).await;

    match file {
        Ok(_) => {
            json!("File removed")
        }
        Err(e) => json!(e.to_string()),
    }
}
