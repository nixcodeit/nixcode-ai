use crate::project::Project;
use glob::glob;
use regex::Regex;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

/// Validates regex pattern and returns a compiled regex or error message
pub fn validate_regex(pattern: &str) -> Result<Regex, serde_json::Value> {
    if pattern.is_empty() {
        return Err(json!("Search pattern is empty"));
    }

    match Regex::new(pattern) {
        Ok(re) => Ok(re),
        Err(e) => Err(json!(format!("Invalid regex pattern: {}", e))),
    }
}

/// Validates glob pattern and returns a resolved path or error message
pub fn validate_and_resolve_glob(
    project: &Arc<Project>,
    glob_pattern: &str,
) -> Result<String, serde_json::Value> {
    if glob_pattern.is_empty() {
        return Err(json!("Glob pattern is empty"));
    }

    let pattern_path_buf = PathBuf::from(glob_pattern);
    if !pattern_path_buf.is_relative() {
        return Err(json!("Glob pattern must be a relative path"));
    }

    match crate::utils::fs::join_path(project.get_cwd(), glob_pattern.to_string()) {
        Ok(p) => Ok(p.to_str().unwrap().to_string()),
        Err(e) => Err(json!(e.to_string())),
    }
}

/// Executes a glob pattern and returns all paths or error message
pub async fn get_glob_paths(pattern_str: String) -> Result<Vec<PathBuf>, serde_json::Value> {
    let glob_result = tokio::task::spawn_blocking(move || glob(&pattern_str))
        .await
        .unwrap();

    match glob_result {
        Ok(paths) => {
            let paths = tokio::task::spawn_blocking(move || {
                paths.filter_map(|p| p.ok()).collect::<Vec<_>>()
            })
            .await
            .unwrap();
            Ok(paths)
        }
        Err(e) => Err(json!(format!("Error processing glob pattern: {}", e))),
    }
}

/// Filters paths based on gitignore and hidden file settings
pub async fn filter_paths(
    project: Arc<Project>,
    paths: Vec<PathBuf>,
    include_hidden: bool,
    include_git: bool,
) -> Vec<(PathBuf, String)> {
    let cwd = project.get_cwd().clone();

    tokio::task::spawn_blocking(move || {
        let repository = git2::Repository::discover(project.get_cwd().as_path()).ok();

        paths
            .iter()
            .filter_map(|path| {
                let result = path.strip_prefix(&cwd);
                if result.is_err() {
                    return None;
                }

                let rel_path = result.unwrap().to_str()?;

                // Check if file should be included based on .git and hidden file filters
                if !include_git && rel_path.contains(".git/") {
                    return None;
                } else if include_git && rel_path.starts_with(".git") {
                    return Some((path.clone(), rel_path.to_string()));
                }

                if !include_hidden && (rel_path.starts_with(".") || rel_path.contains("/.")) {
                    return None;
                }

                // Check gitignore
                if let Some(repo) = &repository {
                    if repo.is_path_ignored(rel_path).unwrap_or(false) {
                        return None;
                    }
                }

                Some((path.clone(), rel_path.to_string()))
            })
            .collect::<Vec<_>>()
    })
    .await
    .unwrap()
}
