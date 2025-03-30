use std::path::PathBuf;
use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::project::Project;

// TODO: Optimize this tool to update part of the file instead of rewriting the whole file
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct UpdateTextFileParams {
    #[schemars(description = "Relative path to file")]
    pub path: String,

    #[schemars(description = "New file content")]
    pub content: String,
}

#[tool("Update file content")]
pub async fn update_text_file(
    params: UpdateTextFileParams,
    project: Arc<Project>,
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

    let file = File::create(&path).await;

    match file {
        Ok(mut f) => {
            f.write_all(params.content.as_bytes()).await.unwrap();
            json!("File updated")
        }
        Err(e) => json!(e.to_string()),
    }
}
