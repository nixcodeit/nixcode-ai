use crate::project::Project;
use glob::glob;
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GlobToolParams {
    #[schemars(description = "Glob pattern")]
    pattern: String,

    #[schemars(description = "Include files excluded by gitignore (default: false)")]
    #[serde(default)]
    include_gitignored: Option<bool>,

    #[schemars(
        description = "Include hidden (prefixed with `.`, like `.github`, `.nixcode` etc) (default: false)"
    )]
    #[serde(default)]
    include_hidden: Option<bool>,

    #[schemars(description = "Offset for search results (default: 0)")]
    #[serde(default)]
    offset: Option<usize>,
}

#[tool("Search for files in project directory using glob pattern")]
pub async fn search_glob_files(params: GlobToolParams, project: Arc<Project>) -> serde_json::Value {
    if params.pattern.is_empty() {
        return json!("Pattern is empty");
    }

    let pattern_path_buf = PathBuf::from(&params.pattern);

    if !pattern_path_buf.is_relative() {
        return json!("Pattern must be relative path");
    }

    let pattern = crate::utils::fs::join_path(project.get_cwd(), params.pattern.clone());
    if pattern.is_err() {
        return json!(pattern.unwrap_err().to_string());
    }

    let pattern = pattern.unwrap();

    let pattern_str = pattern.to_str().unwrap().to_string();
    let glob_result = tokio::task::spawn_blocking(move || glob(&pattern_str))
        .await
        .unwrap();

    let mut result_str = String::new();
    let tool_result = match glob_result {
        Ok(paths) => {
            let include_hidden = params.include_hidden.unwrap_or(false);
            let offset = params.offset.unwrap_or(0);
            let include_git = params.include_gitignored.unwrap_or(false);

            let paths = tokio::task::spawn_blocking(move || {
                paths.filter_map(|p| p.ok()).collect::<Vec<_>>()
            })
            .await
            .unwrap();

            let cwd = project.get_cwd().clone();
            let paths = tokio::task::spawn_blocking(move || {
                let repository = git2::Repository::discover(project.get_cwd().as_path()).ok();

                paths
                    .iter()
                    .map(|path| {
                        let result = path.strip_prefix(&cwd);
                        if let Err(_) = result {
                            return None;
                        }

                        result.unwrap().to_str()
                    })
                    .filter(|path| path.is_some())
                    .map(|path| path.unwrap().to_string())
                    .filter(|path| {
                        if !include_git && path.contains(".git/") {
                            return false;
                        } else if include_git && path.starts_with(".git") {
                            return true;
                        }

                        if !include_hidden && (path.starts_with(".") || path.contains("/.")) {
                            return false;
                        }

                        if let Some(repo) = &repository {
                            if repo.is_path_ignored(path).unwrap_or(false) {
                                return false;
                            }
                        }

                        true
                    })
                    .collect::<Vec<_>>()
            })
            .await
            .unwrap();

            let missing_results = paths.len().saturating_sub(offset + 100);
            paths.iter().skip(offset).take(100).for_each(|path| {
                result_str.push_str(&format!("{}\n", path));
            });

            if result_str.is_empty() {
                return json!("No files found");
            }

            result_str.insert_str(0, "Glob results:\n");

            if missing_results > 0 {
                if offset > 0 {
                    result_str.push_str(&format!("... and {} more files (current offset: {}), reuse tool with offset parameter", missing_results, offset));
                } else {
                    result_str.push_str(&format!(
                        "... and {} more files, reuse tool with offset parameter",
                        missing_results
                    ));
                }
            }

            serde_json::to_value(result_str)
        }
        Err(e) => serde_json::to_value(e.to_string()),
    };

    tool_result.unwrap_or_else(|e| json!(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Project;
    use serde_json::json;
    use std::env::current_dir;

    #[tokio::test]
    async fn test_empty_result() {
        let project = Arc::new(Project::new(PathBuf::from("/tmp")));
        let params = GlobToolParams {
            pattern: "_not_existing_file.xyz.json".to_string(),
            include_gitignored: None,
            offset: None,
            include_hidden: None,
        };

        let result = search_glob_files(params, project).await;
        let expected = json!("No files found");
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_ok_result() {
        let project = Arc::new(Project::new(PathBuf::from(current_dir().unwrap())));
        let params = GlobToolParams {
            pattern: "Cargo.toml".to_string(),
            include_gitignored: None,
            offset: None,
            include_hidden: None,
        };

        let result = search_glob_files(params, project).await;
        let expected = json!("No files found");
        assert_ne!(result, expected);
    }

    #[tokio::test]
    async fn test_ok_git_result() {
        let project = Arc::new(Project::new(PathBuf::from(
            current_dir().unwrap().parent().unwrap().parent().unwrap(),
        )));
        let params = GlobToolParams {
            pattern: ".git/*".to_string(),
            include_gitignored: Some(true),
            offset: None,
            include_hidden: None,
        };

        let result = search_glob_files(params, project).await.to_string();
        let expected = json!("No files found").to_string();
        assert_ne!(result, expected);
    }

    #[cfg(not(target_os = "windows"))]
    #[tokio::test]
    async fn test_ok_hidden_result() {
        let project = Arc::new(Project::new(PathBuf::from(
            current_dir().unwrap().parent().unwrap().parent().unwrap(),
        )));
        let params = GlobToolParams {
            pattern: ".github/*".to_string(),
            include_gitignored: None,
            offset: None,
            include_hidden: Some(true),
        };

        let result = search_glob_files(params, project).await.to_string();
        let expected = json!("No files found").to_string();
        assert_ne!(result, expected);
        assert!(result.contains(".github/ISSUE_TEMPLATE"));
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn test_ok_hidden_result() {
        let project = Arc::new(Project::new(PathBuf::from(
            current_dir().unwrap().parent().unwrap().parent().unwrap(),
        )));
        let params = GlobToolParams {
            pattern: ".github/*".to_string(),
            include_gitignored: None,
            offset: None,
            include_hidden: Some(true),
        };

        let result = search_glob_files(params, project).await.to_string();
        let expected = json!("No files found").to_string();
        assert_ne!(result, expected);
        assert!(result.contains(".github\\ISSUE_TEMPLATE"));
    }

    #[tokio::test]
    async fn test_ok_not_git_result() {
        let project = Arc::new(Project::new(PathBuf::from(
            current_dir().unwrap().parent().unwrap().parent().unwrap(),
        )));
        let params = GlobToolParams {
            pattern: ".git/*".to_string(),
            include_gitignored: None,
            offset: None,
            include_hidden: None,
        };

        let result = search_glob_files(params, project).await.to_string();
        let expected = json!("No files found").to_string();
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_many_files() {
        let project = Arc::new(Project::new(PathBuf::from(
            current_dir().unwrap().parent().unwrap().parent().unwrap(),
        )));
        let params = GlobToolParams {
            pattern: "**/*".to_string(),
            include_gitignored: Some(true),
            offset: None,
            include_hidden: None,
        };

        let result = search_glob_files(params, project).await.to_string();

        assert!(result
            .to_string()
            .contains("reuse tool with offset parameter"));
    }

    #[tokio::test]
    async fn test_many_files_offset() {
        let project = Arc::new(Project::new(PathBuf::from(
            current_dir().unwrap().parent().unwrap().parent().unwrap(),
        )));
        let params = GlobToolParams {
            pattern: "**/*".to_string(),
            include_gitignored: Some(true),
            offset: Some(100),
            include_hidden: None,
        };

        let result = search_glob_files(params, project).await.to_string();

        assert!(result.contains("current offset: 100"));
    }
}
