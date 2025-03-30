use super::*;
use crate::project::Project;
use serde_json::json;
use std::env::current_dir;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::test]
async fn test_empty_pattern() {
    let project = Arc::new(Project::new(PathBuf::from("/tmp")));
    let params = search_content::SearchContentParams {
        pattern: "".to_string(),
        glob_pattern: "**/*.rs".to_string(),
        include_gitignored: None,
        include_hidden: None,
        offset: None,
    };

    let result = search_content::search_content(params, project).await;
    let expected = json!("Search pattern is empty");
    assert_eq!(result, expected);
}

#[tokio::test]
async fn test_empty_glob() {
    let project = Arc::new(Project::new(PathBuf::from("/tmp")));
    let params = search_content::SearchContentParams {
        pattern: "test".to_string(),
        glob_pattern: "".to_string(),
        include_gitignored: None,
        include_hidden: None,
        offset: None,
    };

    let result = search_content::search_content(params, project).await;
    let expected = json!("Glob pattern is empty");
    assert_eq!(result, expected);
}

#[tokio::test]
async fn test_invalid_regex() {
    let project = Arc::new(Project::new(PathBuf::from("/tmp")));
    let params = search_content::SearchContentParams {
        pattern: "[invalid".to_string(),
        glob_pattern: "**/*.rs".to_string(),
        include_gitignored: None,
        include_hidden: None,
        offset: None,
    };

    let result = search_content::search_content(params, project)
        .await
        .to_string();
    assert!(result.contains("Invalid regex pattern"));
}

#[tokio::test]
async fn test_search_content() {
    let project = Arc::new(Project::new(PathBuf::from(
        current_dir().unwrap().parent().unwrap().parent().unwrap(),
    )));
    let params = search_content::SearchContentParams {
        pattern: "SearchContentParams".to_string(), // Should find itself
        glob_pattern: "**/*.rs".to_string(),
        include_gitignored: None,
        include_hidden: None,
        offset: None,
    };

    let result = search_content::search_content(params, project)
        .await
        .to_string();
    assert!(result.contains("Found"));
    assert!(result.contains("SearchContentParams"));
}
