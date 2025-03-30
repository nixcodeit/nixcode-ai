use crate::project::Project;
use glob::glob;
use nixcode_macros::tool;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct SearchContentParams {
    #[schemars(description = "Regex pattern to search for in file content")]
    pattern: String,

    #[schemars(description = "Glob pattern for files to search in")]
    glob_pattern: String,

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

#[derive(Serialize)]
struct SearchMatch {
    path: String,
    line_number: usize,
    line_content: String,
}

#[tool("Search for text content in files using regex pattern")]
pub async fn search_content(
    params: SearchContentParams,
    project: Arc<Project>,
) -> serde_json::Value {
    if params.pattern.is_empty() {
        return json!("Search pattern is empty");
    }

    if params.glob_pattern.is_empty() {
        return json!("Glob pattern is empty");
    }

    // Compile regex pattern
    let regex = match Regex::new(&params.pattern) {
        Ok(re) => re,
        Err(e) => return json!(format!("Invalid regex pattern: {}", e)),
    };

    // Create glob pattern
    let pattern_path_buf = PathBuf::from(&params.glob_pattern);
    if !pattern_path_buf.is_relative() {
        return json!("Glob pattern must be a relative path");
    }

    let pattern = match crate::utils::fs::join_path(project.get_cwd(), params.glob_pattern.clone())
    {
        Ok(p) => p,
        Err(e) => return json!(e.to_string()),
    };

    let pattern_str = pattern.to_str().unwrap().to_string();

    // Get matching files using glob
    let glob_result = tokio::task::spawn_blocking(move || glob(&pattern_str))
        .await
        .unwrap();

    match glob_result {
        Ok(paths) => {
            let include_hidden = params.include_hidden.unwrap_or(false);
            let include_git = params.include_gitignored.unwrap_or(false);
            let offset = params.offset.unwrap_or(0);
            const LIMIT: usize = 100;

            // Get all file paths
            let paths = tokio::task::spawn_blocking(move || {
                paths.filter_map(|p| p.ok()).collect::<Vec<_>>()
            })
            .await
            .unwrap();

            // Filter paths based on gitignore and hidden files settings
            let cwd = project.get_cwd().clone();
            let paths = tokio::task::spawn_blocking(move || {
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

                        if !include_hidden && (rel_path.starts_with(".") || rel_path.contains("/."))
                        {
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
            .unwrap();

            // Search the files for content matches
            let regex_pattern = regex.clone();
            let results = tokio::task::spawn_blocking(move || {
                let mut matches = Vec::new();
                let mut total_matches = 0;

                for (file_path, rel_path) in paths {
                    // Skip directories
                    if file_path.is_dir() {
                        continue;
                    }

                    // Try to open the file
                    let file = match File::open(&file_path) {
                        Ok(f) => f,
                        Err(_) => continue,
                    };

                    // Read the file line by line and check for matches
                    let reader = BufReader::new(file);
                    for (line_num, line_result) in reader.lines().enumerate() {
                        // Skip lines that can't be read
                        let line = match line_result {
                            Ok(l) => l,
                            Err(_) => continue,
                        };

                        // Check if line matches regex
                        if regex_pattern.is_match(&line) {
                            total_matches += 1;

                            // Skip matches before the offset
                            if total_matches <= offset {
                                continue;
                            }

                            matches.push(SearchMatch {
                                path: rel_path.clone(),
                                line_number: line_num + 1, // 1-based line numbers
                                line_content: line.trim().to_string(),
                            });

                            // Check if we've reached the limit
                            if matches.len() >= LIMIT {
                                break;
                            }
                        }
                    }

                    // Stop if we've reached the limit
                    if matches.len() >= LIMIT {
                        break;
                    }
                }

                (matches, total_matches)
            })
            .await
            .unwrap();

            // Format the results
            if results.0.is_empty() {
                json!("No matches found")
            } else {
                let (matches, total_matches) = results;
                let mut result_str = format!(
                    "Found {} matches for pattern '{}' in files matching '{}':\n\n",
                    total_matches, params.pattern, params.glob_pattern
                );

                for m in &matches {
                    result_str.push_str(&format!(
                        "{}:{}: {}\n",
                        m.path, m.line_number, m.line_content
                    ));
                }

                let missing_results = total_matches.saturating_sub(offset + matches.len());
                if missing_results > 0 {
                    if offset > 0 {
                        result_str.push_str(&format!("\n... and {} more matches (current offset: {}), reuse tool with offset parameter", missing_results, offset));
                    } else {
                        result_str.push_str(&format!(
                            "\n... and {} more matches, reuse tool with offset parameter",
                            missing_results
                        ));
                    }
                }

                json!(result_str)
            }
        }
        Err(e) => json!(format!("Error processing glob pattern: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Project;
    use serde_json::json;
    use std::env::current_dir;

    #[tokio::test]
    async fn test_empty_pattern() {
        let project = Arc::new(Project::new(PathBuf::from("/tmp")));
        let params = SearchContentParams {
            pattern: "".to_string(),
            glob_pattern: "**/*.rs".to_string(),
            include_gitignored: None,
            include_hidden: None,
            offset: None,
        };

        let result = search_content(params, project).await;
        let expected = json!("Search pattern is empty");
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_empty_glob() {
        let project = Arc::new(Project::new(PathBuf::from("/tmp")));
        let params = SearchContentParams {
            pattern: "test".to_string(),
            glob_pattern: "".to_string(),
            include_gitignored: None,
            include_hidden: None,
            offset: None,
        };

        let result = search_content(params, project).await;
        let expected = json!("Glob pattern is empty");
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_invalid_regex() {
        let project = Arc::new(Project::new(PathBuf::from("/tmp")));
        let params = SearchContentParams {
            pattern: "[invalid".to_string(),
            glob_pattern: "**/*.rs".to_string(),
            include_gitignored: None,
            include_hidden: None,
            offset: None,
        };

        let result = search_content(params, project).await.to_string();
        assert!(result.contains("Invalid regex pattern"));
    }

    #[tokio::test]
    async fn test_search_content() {
        let project = Arc::new(Project::new(PathBuf::from(
            current_dir().unwrap().parent().unwrap().parent().unwrap(),
        )));
        dbg!(&project);
        let params = SearchContentParams {
            pattern: "SearchContentParams".to_string(), // Should find itself
            glob_pattern: "**/*.rs".to_string(),
            include_gitignored: None,
            include_hidden: None,
            offset: None,
        };

        let result = search_content(params, project).await.to_string();
        dbg!(&result);
        assert!(result.contains("Found"));
        assert!(result.contains("SearchContentParams"));
    }
}
