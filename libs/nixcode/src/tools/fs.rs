use crate::project::Project;
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct CreateFileParams {
    #[schemars(description = "Relative path to new file")]
    path: String,
}

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct ReadTextFileParams {
    #[schemars(description = "Relative path to file")]
    path: String,
}

// TODO: Optimize this tool to update part of the file instead of rewriting the whole file
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct UpdateTextFileParams {
    #[schemars(description = "Relative path to file")]
    path: String,

    #[schemars(description = "New file content")]
    content: String,
}

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct DeleteFileParams {
    #[schemars(description = "Relative path to file")]
    path: String,
}

#[tool("Create empty file in given path")]
pub async fn create_file(params: CreateFileParams, project: &Project) -> serde_json::Value {
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    use crate::utils::fs;

    let file_path = PathBuf::from(params.path);

    let cwd = project.get_cwd();
    let path = fs::join_path(cwd.clone(), file_path);
    if path.is_err() {
        return serde_json::json!(path.unwrap_err().to_string());
    }

    let path = path.unwrap();
    if !path.starts_with(cwd) {
        return serde_json::json!("Path must be inside project directory");
    }

    // create directories if they don't exist
    let parent = path.parent().unwrap();
    let create_dirs_result = tokio::fs::create_dir_all(parent).await;
    if create_dirs_result.is_err() {
        return serde_json::json!(create_dirs_result.unwrap_err().to_string());
    }

    let file = File::create(&path).await;

    match file {
        Ok(mut f) => {
            f.write_all(b"").await.unwrap();
            serde_json::json!("File created")
        }
        Err(e) => serde_json::json!(e.to_string())
    }
}

#[tool("Read file content")]
pub async fn read_text_file(params: ReadTextFileParams, project: &Project) -> serde_json::Value {
    use tokio::fs::read_to_string;
    use crate::utils::fs;

    let file_path = PathBuf::from(params.path);

    let cwd = project.get_cwd();
    let path = fs::join_path(cwd.clone(), file_path);
    if path.is_err() {
        return serde_json::json!(path.unwrap_err().to_string());
    }

    let path = path.unwrap();
    if !path.starts_with(cwd) {
        return serde_json::json!("Path must be inside project directory");
    }

    let file = read_to_string(&path).await;

    match file {
        Ok(content) => {
            serde_json::json!(content)
        }
        Err(e) => serde_json::json!(e.to_string())
    }
}

#[tool("Update file content")]
pub async fn update_text_file(params: UpdateTextFileParams, project: &Project) -> serde_json::Value {
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    use crate::utils::fs;

    let file_path = PathBuf::from(params.path);

    let cwd = project.get_cwd();
    let path = fs::join_path(cwd.clone(), file_path);
    if path.is_err() {
        return serde_json::json!(path.unwrap_err().to_string());
    }

    let path = path.unwrap();
    if !path.starts_with(cwd) {
        return serde_json::json!("Path must be inside project directory");
    }

    let file = File::create(&path).await;

    match file {
        Ok(mut f) => {
            f.write_all(params.content.as_bytes()).await.unwrap();
            serde_json::json!("File updated")
        }
        Err(e) => serde_json::json!(e.to_string())
    }
}

#[tool("Delete file")]
pub async fn delete_file(params: DeleteFileParams, project: &Project) -> serde_json::Value {
    use tokio::fs::remove_file;
    use crate::utils::fs;

    let file_path = PathBuf::from(params.path);

    let cwd = project.get_cwd();
    let path = fs::join_path(cwd.clone(), file_path);
    if path.is_err() {
        return serde_json::json!(path.unwrap_err().to_string());
    }

    let path = path.unwrap();
    if !path.starts_with(cwd) {
        return serde_json::json!("Path must be inside project directory");
    }

    let file = remove_file(&path).await;

    match file {
        Ok(_) => {
            serde_json::json!("File removed")
        }
        Err(e) => serde_json::json!(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::fs::join_path;

    #[test]
    fn test_join_path() {
        let base = PathBuf::from("tmp");
        let path = PathBuf::from("./../foo.txt");

        let result = join_path(base, path).unwrap();

        assert_eq!(result, PathBuf::from("foo.txt"));
    }

    #[test]
    fn test_join_path_absolute() {
        let base = PathBuf::from("tmp");
        let path = PathBuf::from("/foo.txt");

        let result = join_path(base, path);

        assert!(result.is_err());
    }

    #[test]
    fn test_join_path_multiple_relatives() {
        let base = PathBuf::from("/tmp");
        let path = PathBuf::from("./../../../../foo.txt");

        let result = join_path(base, path);

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_no_create_file() {
        let project = Project::new(PathBuf::from("/tmp"));
        let params = CreateFileParams {
            path: String::from("./../foo.txt"),
        };

        let result = create_file(params, &project).await;

        assert_eq!(result, serde_json::json!("Path must be inside project directory"));
    }

    #[tokio::test]
    async fn test_create_file() {
        let project = Project::new(PathBuf::from("/tmp"));
        let params = CreateFileParams {
            path: String::from("foo.txt"),
        };

        let result = create_file(params, &project).await;

        assert_eq!(result, serde_json::json!("File created"));
    }
}