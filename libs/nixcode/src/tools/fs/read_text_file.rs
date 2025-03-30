use std::path::PathBuf;
use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct ReadTextFileParams {
    #[schemars(description = "Relative path to file")]
    pub path: String,
}

#[tool("Read file content")]
pub async fn read_text_file(
    params: ReadTextFileParams,
    project: Arc<Project>,
) -> serde_json::Value {
    use crate::utils::fs;
    use tokio::fs::read_to_string;

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

    let file = read_to_string(&path).await;

    match file {
        Ok(content) => {
            json!(content)
        }
        Err(e) => json!(e.to_string()),
    }
}
