use std::ffi::OsStr;
use std::path::PathBuf;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use nixcode_macros::tool;
use crate::project::Project;

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
pub fn create_file(params: CreateFileParams, project: &Project) -> serde_json::Value {
    use std::fs::File;
    use std::io::Write;

    let file_path = PathBuf::from(params.path);

    let cwd = project.get_cwd();
    let path = join_path(cwd.clone(), file_path);
    if path.is_err() {
        return serde_json::json!(path.unwrap_err().to_string());
    }

    let path = path.unwrap();
    if !path.starts_with(cwd) {
        return serde_json::json!("Path must be inside project directory");
    }

    let file = File::create(&path);

    match file {
        Ok(mut f) => {
            f.write_all(b"").unwrap();
            serde_json::json!("File created")
        }
        Err(e) => serde_json::json!(e.to_string())
    }
}

#[tool("Read file content")]
pub fn read_text_file(params: ReadTextFileParams, project: &Project) -> serde_json::Value {
    use std::fs::read_to_string;

    let file_path = PathBuf::from(params.path);

    let cwd = project.get_cwd();
    let path = join_path(cwd.clone(), file_path);
    if path.is_err() {
        return serde_json::json!(path.unwrap_err().to_string());
    }

    let path = path.unwrap();
    if !path.starts_with(cwd) {
        return serde_json::json!("Path must be inside project directory");
    }

    let file = read_to_string(&path);

    match file {
        Ok(content) => {
            serde_json::json!(content)
        }
        Err(e) => serde_json::json!(e.to_string())
    }
}

#[tool("Update file content")]
fn update_text_file(params: UpdateTextFileParams, project: &Project) -> serde_json::Value {
    use std::fs::File;
    use std::io::Write;

    let file_path = PathBuf::from(params.path);

    let cwd = project.get_cwd();
    let path = join_path(cwd.clone(), file_path);
    if path.is_err() {
        return serde_json::json!(path.unwrap_err().to_string());
    }

    let path = path.unwrap();
    if !path.starts_with(cwd) {
        return serde_json::json!("Path must be inside project directory");
    }

    let file = File::create(&path);

    match file {
        Ok(mut f) => {
            f.write_all(params.content.as_bytes()).unwrap();
            serde_json::json!("File updated")
        }
        Err(e) => serde_json::json!(e.to_string())
    }
}

#[tool("Delete file")]
fn delete_file(params: DeleteFileParams, project: &Project) -> serde_json::Value {
    use std::fs::remove_file;

    let file_path = PathBuf::from(params.path);

    let cwd = project.get_cwd();
    let path = join_path(cwd.clone(), file_path);
    if path.is_err() {
        return serde_json::json!(path.unwrap_err().to_string());
    }

    let path = path.unwrap();
    if !path.starts_with(cwd) {
        return serde_json::json!("Path must be inside project directory");
    }

    let file = remove_file(&path);

    match file {
        Ok(_) => {
            serde_json::json!("File removed")
        }
        Err(e) => serde_json::json!(e.to_string())
    }
}

fn join_path(base: impl Into<PathBuf>, path: impl Into<PathBuf>) -> anyhow::Result<PathBuf> {
    let path = path.into();
    let mut base = base.into();

    if path.is_absolute() {
        return Err(anyhow::anyhow!("Path must be relative"));
    }

    for part in path.iter() {
        if part == OsStr::new("..") {
            if !base.pop() {
                return Err(anyhow::anyhow!("Path exceeds base directory"));
            }
        } else if part != OsStr::new(".") && part != OsStr::new("/") {
            base.push(part);
        }
    }

    Ok(base)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_no_create_file() {
        let project = Project::new(PathBuf::from("/tmp"));
        let params = CreateFileParams {
            path: String::from("./../foo.txt"),
        };

        let result = create_file(params, &project);

        assert_eq!(result, serde_json::json!("Path must be inside project directory"));
    }

    #[test]
    fn test_create_file() {
        let project = Project::new(PathBuf::from("/tmp"));
        let params = CreateFileParams {
            path: String::from("foo.txt"),
        };

        let result = create_file(params, &project);

        assert_eq!(result, serde_json::json!("File created"));
    }
}