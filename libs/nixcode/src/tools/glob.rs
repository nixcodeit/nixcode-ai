use std::path::{PathBuf, MAIN_SEPARATOR};
use glob::glob;
use schemars::{json_schema, schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;
use nixcode_macros::{tool};
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GlobToolParams {
    #[schemars(description = "Glob pattern")]
    pattern: String,
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

    if pattern_path_buf.starts_with("..") {
        return json!("Pattern must not start with '..'");
    }

    let pattern = format!("{}/{}", project.get_cwd().to_str().unwrap_or("."), params.pattern);
    let result = glob(pattern.as_str());

    let mut result_str = String::new();
    let tool_result = match result {
        Ok(paths) => {
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

                result_str.push_str(&format!("{:?}\n", result.unwrap()));
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
        };

        let result = search_glob_files(params, &project);
        let expected = json!("No files found");
        assert_ne!(result, expected);
    }
}