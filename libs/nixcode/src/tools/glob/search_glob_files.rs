use crate::project::Project;
use glob::glob;
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GlobToolParams {
    #[schemars(description = "Glob pattern")]
    pub pattern: String,

    #[schemars(description = "Include files excluded by gitignore (default: false)")]
    #[serde(default)]
    pub include_gitignored: Option<bool>,

    #[schemars(
        description = "Include hidden (prefixed with `.`, like `.github`, `.nixcode` etc) (default: false)"
    )]
    #[serde(default)]
    pub include_hidden: Option<bool>,

    #[schemars(description = "Offset for search results (default: 0)")]
    #[serde(default)]
    pub offset: Option<usize>,
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
                // Check if we're in a git repository
                let is_git_repo = Command::new("git")
                    .current_dir(&project.get_cwd())
                    .args(["rev-parse", "--is-inside-work-tree"])
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false);

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

                        // Check if file is ignored by git
                        if is_git_repo && !include_git {
                            let output = Command::new("git")
                                .current_dir(&cwd)
                                .args(["check-ignore", "-q", path])
                                .output();
                            
                            if let Ok(output) = output {
                                // If the command succeeds (exit code 0), the file is ignored
                                if output.status.success() {
                                    return false;
                                }
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