use super::*;
use crate::project::Project;
use crate::utils::fs::join_path;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn test_join_path() {
    let base = PathBuf::from("tmp");
    let path = PathBuf::from("./../foo.txt");

    let result = join_path(base, path).unwrap();

    assert_eq!(result, PathBuf::from("foo.txt"));
}

#[cfg(not(target_os = "windows"))]
#[test]
fn test_join_path_absolute() {
    let base = PathBuf::from("tmp");
    let path = PathBuf::from("/foo.txt");

    let result = join_path(base, path);

    assert!(result.is_err());
}

#[cfg(target_os = "windows")]
#[test]
fn test_join_path_absolute() {
    let base = PathBuf::from("tmp");
    let path = PathBuf::from("C:\\foo.txt");

    let result = join_path(base, path);

    assert!(result.is_err());
}

#[cfg(not(target_os = "windows"))]
#[test]
fn test_join_path_multiple_relatives() {
    let base = PathBuf::from("/tmp");
    let path = PathBuf::from("./../../../../foo.txt");

    let result = join_path(base, path);

    assert!(result.is_err());
}

#[cfg(target_os = "windows")]
#[test]
fn test_join_path_multiple_relatives() {
    let base = PathBuf::from("C:\\tmp");
    let path = PathBuf::from("./../../../../foo.txt");

    let result = join_path(base, path);

    assert!(result.is_err());
}

#[tokio::test]
async fn test_no_create_file() {
    let project = Arc::new(Project::new(PathBuf::from("/tmp")));
    let params = create_file::CreateFileParams {
        path: String::from("./../foo.txt"),
    };

    let result = create_file::create_file(params, project).await;

    assert_eq!(
        result,
        serde_json::json!("Path must be inside project directory")
    );
}

#[tokio::test]
async fn test_create_file() {
    let project = Arc::new(Project::new(PathBuf::from("/tmp")));
    let params = create_file::CreateFileParams {
        path: String::from("foo.txt"),
    };

    let result = create_file::create_file(params, project).await;

    assert_eq!(result, serde_json::json!("File created"));
}
