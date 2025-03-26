use crate::project::Project;
use glob::glob;
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GlobToolParams {
    #[schemars(description = "Glob pattern")]
    pattern: String,

    #[schemars(description = "Include .git directory in search (default: false)")]
    #[serde(default)]
    include_git: Option<bool>,

    #[schemars(description = "Offset for search results (default: 0)")]
    #[serde(default)]
    offset: Option<usize>,
}

#[tool("Search for files in project directory using glob pattern")]
pub fn search_glob_files(params: GlobToolParams, project: &Project) -> serde_json::Value {
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

    let offset = params.offset.unwrap_or(0);
    let pattern = pattern.unwrap();

    let result = glob(pattern.to_str().unwrap());

    let mut result_str = String::new();
    let tool_result = match result {
        Ok(paths) => {
            let include_git = params.include_git.unwrap_or(false);
            
            let paths = paths
                .filter(Result::is_ok)
                .map(|p| p.unwrap())
                .collect::<Vec<_>>();

            if paths.is_empty() {
                return json!("No files found");
            }

            result_str.push_str("Glob results:\n");
            let paths = paths.iter().map(|path| {
                let result = path.strip_prefix(project.get_cwd());
                if result.is_err() {
                    return None;
                }

                result.unwrap().to_str()
            })
                .filter(|path| path.is_some())
                .map(|path| path.unwrap().to_string())
                .filter(|path| {
                    if !include_git && (path.starts_with(".git") || path.contains(".git/")) {
                        return false;
                    } else if include_git && path.starts_with(".git") {
                        return true;
                    }

                    !(path.starts_with(".")
                        || path.contains("/.")
                        || path.starts_with("target/")
                        || path.contains("node_modules/"))
                }).collect::<Vec<_>>();

            let missing_results = paths.len().saturating_sub(offset + 100);
            paths.iter().skip(offset).take(100).for_each(|path| {
                result_str.push_str(&format!("{}\n", path));
            });

            if missing_results > 0 {
                if offset > 0 {
                    result_str.push_str(&format!("... and {} more files (current offset: {}), reuse tool with offset parameter", missing_results, offset));
                } else {
                    result_str.push_str(&format!("... and {} more files, reuse tool with offset parameter", missing_results));
                }
            }

            serde_json::to_value(result_str)
        }
        Err(e) => serde_json::to_value(e.to_string())
    };

    tool_result.unwrap_or_else(|e| json!(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Project;
    use serde_json::json;
    use std::env::current_dir;

    #[test]
    fn test_empty_result() {
        let project = Project::new(PathBuf::from("/tmp"));
        let params = GlobToolParams {
            pattern: "_not_existing_file.xyz.json".to_string(),
            include_git: None,
            offset: None,
        };

        let result = search_glob_files(params, &project);
        let expected = json!("No files found");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_ok_result() {
        let project = Project::new(PathBuf::from(current_dir().unwrap()));
        let params = GlobToolParams {
            pattern: "Cargo.toml".to_string(),
            include_git: None,
            offset: None,
        };

        let result = search_glob_files(params, &project);
        let expected = json!("No files found");
        assert_ne!(result, expected);
    }

    #[test]
    fn test_ok_git_result() {
        let project = Project::new(PathBuf::from(current_dir().unwrap()));
        let params = GlobToolParams {
            pattern: "../../.git/*".to_string(),
            include_git: Some(true),
            offset: None,
        };

        let result = search_glob_files(params, &project).to_string();
        let expected = json!("No files found").to_string();
        assert_ne!(result, expected);
    }

    #[test]
    fn test_ok_not_git_result() {
        let project = Project::new(PathBuf::from(current_dir().unwrap()));
        let params = GlobToolParams {
            pattern: ".git/*".to_string(),
            include_git: None,
            offset: None,
        };

        let result = search_glob_files(params, &project).to_string();
        let expected = json!("No files found").to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_many_files() {
        let project = Project::new(PathBuf::from("/Users/nix/GIT/nixcode-ai"));
        let params = GlobToolParams {
            pattern: "**/*".to_string(),
            include_git: Some(true),
            offset: None,
        };

        let result = search_glob_files(params, &project).to_string();

        assert!(result.to_string().contains("reuse tool with offset parameter"));
    }

    #[test]
    fn test_many_files_offset() {
        let project = Project::new(PathBuf::from("/Users/nix/GIT/nixcode-ai"));
        let params = GlobToolParams {
            pattern: "**/*".to_string(),
            include_git: Some(true),
            offset: Some(100),
        };

        let result = search_glob_files(params, &project).to_string();

        assert!(result.to_string().contains("current offset: 100"));
    }
}