use std::path::PathBuf;
use glob::glob;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use nixcode_macros::{tool};
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GlobToolParams {
    #[schemars(description = "Glob pattern")]
    pattern: String,

    #[schemars(description = "Include .git directory in search (default: false)")]
    #[serde(default)]
    include_git: Option<bool>,
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
            for path in paths {
                let result = path.strip_prefix(project.get_cwd());
                if result.is_err() {
                    return json!(result.unwrap_err().to_string());
                }
                let result = result.unwrap().to_str();

                if let Some(path) = result {
                    if !include_git && (path.starts_with(".git") || path.contains(".git/")) {
                        continue;
                    }
                    // TODO: change to more elegant way, read gitignore or smth
                    if path.starts_with(".")
                        || path.contains("/.")
                        || path.starts_with("target/")
                        || path.contains("node_modules/") {
                        continue;
                    }
                    result_str.push_str(&format!("{}\n", path));
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
    use std::env::current_dir;
    use super::*;
    use crate::project::Project;
    use serde_json::json;

    #[test]
    fn test_empty_result() {
        let project = Project::new(PathBuf::from("/tmp"));
        let params = GlobToolParams {
            pattern: "_not_existing_file.xyz.json".to_string(),
            include_git: None,
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
        };

        let result = search_glob_files(params, &project).to_string();
        let expected = json!("No files found").to_string();
        assert_eq!(result, expected);
    }
}