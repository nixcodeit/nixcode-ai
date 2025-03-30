use super::*;
use crate::project::Project;
use serde_json::json;
use std::env::current_dir;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::test]
async fn test_empty_result() {
    let project = Arc::new(Project::new(PathBuf::from("/tmp")));
    let params = search_glob_files::GlobToolParams {
        pattern: "_not_existing_file.xyz.json".to_string(),
        include_gitignored: None,
        offset: None,
        include_hidden: None,
    };

    let result = search_glob_files::search_glob_files(params, project).await;
    let expected = json!("No files found");
    assert_eq!(result, expected);
}

#[tokio::test]
async fn test_ok_result() {
    let project = Arc::new(Project::new(PathBuf::from(current_dir().unwrap())));
    let params = search_glob_files::GlobToolParams {
        pattern: "Cargo.toml".to_string(),
        include_gitignored: None,
        offset: None,
        include_hidden: None,
    };

    let result = search_glob_files::search_glob_files(params, project).await;
    let expected = json!("No files found");
    assert_ne!(result, expected);
}

#[tokio::test]
async fn test_ok_git_result() {
    let project = Arc::new(Project::new(PathBuf::from(
        current_dir().unwrap().parent().unwrap().parent().unwrap(),
    )));
    let params = search_glob_files::GlobToolParams {
        pattern: ".git/*".to_string(),
        include_gitignored: Some(true),
        offset: None,
        include_hidden: None,
    };

    let result = search_glob_files::search_glob_files(params, project)
        .await
        .to_string();
    let expected = json!("No files found").to_string();
    assert_ne!(result, expected);
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn test_ok_hidden_result() {
    let project = Arc::new(Project::new(PathBuf::from(
        current_dir().unwrap().parent().unwrap().parent().unwrap(),
    )));
    let params = search_glob_files::GlobToolParams {
        pattern: ".github/*".to_string(),
        include_gitignored: None,
        offset: None,
        include_hidden: Some(true),
    };

    let result = search_glob_files::search_glob_files(params, project)
        .await
        .to_string();
    let expected = json!("No files found").to_string();
    assert_ne!(result, expected);
    assert!(result.contains(".github/ISSUE_TEMPLATE"));
}

#[tokio::test]
async fn test_ok_not_git_result() {
    let project = Arc::new(Project::new(PathBuf::from(
        current_dir().unwrap().parent().unwrap().parent().unwrap(),
    )));
    let params = search_glob_files::GlobToolParams {
        pattern: ".git/*".to_string(),
        include_gitignored: None,
        offset: None,
        include_hidden: None,
    };

    let result = search_glob_files::search_glob_files(params, project)
        .await
        .to_string();
    let expected = json!("No files found").to_string();
    assert_eq!(result, expected);
}

#[tokio::test]
async fn test_many_files() {
    let project = Arc::new(Project::new(PathBuf::from(
        current_dir().unwrap().parent().unwrap().parent().unwrap(),
    )));
    let params = search_glob_files::GlobToolParams {
        pattern: "**/*".to_string(),
        include_gitignored: Some(true),
        offset: None,
        include_hidden: None,
    };

    let result = search_glob_files::search_glob_files(params, project)
        .await
        .to_string();

    assert!(result
        .to_string()
        .contains("reuse tool with offset parameter"));
}

#[tokio::test]
async fn test_many_files_offset() {
    let project = Arc::new(Project::new(PathBuf::from(
        current_dir().unwrap().parent().unwrap().parent().unwrap(),
    )));
    let params = search_glob_files::GlobToolParams {
        pattern: "**/*".to_string(),
        include_gitignored: Some(true),
        offset: Some(100),
        include_hidden: None,
    };

    let result = search_glob_files::search_glob_files(params, project)
        .await
        .to_string();

    assert!(result.contains("current offset: 100"));
}
