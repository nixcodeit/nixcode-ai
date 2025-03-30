use crate::project::Project;
use crate::tools::search::content_utils::{
    filter_paths, get_glob_paths, validate_and_resolve_glob, validate_regex,
};
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct SearchContentParams {
    #[schemars(description = "Regex pattern to search for in file content")]
    pub pattern: String,

    #[schemars(description = "Glob pattern for files to search in")]
    pub glob_pattern: String,

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
    // Validate regex pattern
    let regex = match validate_regex(&params.pattern) {
        Ok(re) => re,
        Err(e) => return e,
    };

    // Validate and resolve glob pattern
    let pattern_str = match validate_and_resolve_glob(&project, &params.glob_pattern) {
        Ok(p) => p,
        Err(e) => return e,
    };

    // Get matching files using glob
    let paths = match get_glob_paths(pattern_str).await {
        Ok(p) => p,
        Err(e) => return e,
    };

    // Parse options
    let include_hidden = params.include_hidden.unwrap_or(false);
    let include_git = params.include_gitignored.unwrap_or(false);
    let offset = params.offset.unwrap_or(0);
    const LIMIT: usize = 100;

    // Filter paths based on gitignore and hidden files settings
    let filtered_paths = filter_paths(project.clone(), paths, include_hidden, include_git).await;

    // Search the files for content matches
    let regex_pattern = regex.clone();
    let results = tokio::task::spawn_blocking(move || {
        let mut matches = Vec::new();
        let mut total_matches = 0;

        for (file_path, rel_path) in filtered_paths {
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
