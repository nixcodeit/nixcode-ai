use crate::project::Project;
use crate::tools::content_utils::{
    filter_paths, get_glob_paths, validate_and_resolve_glob, validate_regex,
};
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::sync::Arc;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct ReplaceContentParams {
    #[schemars(description = "Regex pattern to search for in file content")]
    pattern: String,

    #[schemars(description = "Replacement string (can use regex capture groups like $1, $2)")]
    replacement: String,

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
}

#[derive(Serialize)]
struct ReplacementResult {
    path: String,
    matches: usize,
}

#[tool("Replace text content in files based on regex pattern")]
pub async fn replace_content(
    params: ReplaceContentParams,
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

    // Filter paths based on gitignore and hidden files settings
    let filtered_paths = filter_paths(project.clone(), paths, include_hidden, include_git).await;

    // Process replacements in files
    let regex_pattern = regex.clone();
    let replacement = params.replacement.clone();
    let results = tokio::task::spawn_blocking(move || {
        let mut replacement_results = Vec::new();
        let mut total_files_changed = 0;
        let mut total_replacements = 0;
        let mut skipped_files = 0;

        for (file_path, rel_path) in filtered_paths {
            // Skip directories
            if file_path.is_dir() {
                continue;
            }

            // Try to open the file
            let file = match File::open(&file_path) {
                Ok(f) => f,
                Err(_) => {
                    skipped_files += 1;
                    continue;
                }
            };

            // Read the entire file content
            let reader = BufReader::new(file);
            let lines: Result<Vec<String>, _> = reader.lines().collect();
            let lines = match lines {
                Ok(l) => l,
                Err(_) => {
                    skipped_files += 1;
                    continue;
                }
            };

            // Check and perform replacements
            let mut file_changed = false;
            let mut matches_in_file = 0;
            let new_lines: Vec<String> = lines
                .into_iter()
                .map(|line| {
                    if regex_pattern.is_match(&line) {
                        let new_line = regex_pattern.replace_all(&line, &replacement).to_string();
                        if new_line != line {
                            file_changed = true;
                            matches_in_file += 1;
                            total_replacements += 1;
                            return new_line;
                        }
                    }
                    line
                })
                .collect();

            // If replacements were made, write the changes back to the file
            if file_changed {
                replacement_results.push(ReplacementResult {
                    path: rel_path.clone(),
                    matches: matches_in_file,
                });

                total_files_changed += 1;

                // Write the updated content back to the file
                if let Err(_) = fs::write(&file_path, new_lines.join("\n")) {
                    // If writing fails, increment skipped files counter
                    skipped_files += 1;
                    continue;
                }
            }
        }

        (
            replacement_results,
            total_files_changed,
            total_replacements,
            skipped_files,
        )
    })
    .await
    .unwrap();

    // Format the results
    let (replacement_results, total_files_changed, total_replacements, skipped_files) = results;

    if total_replacements == 0 {
        json!("No matches found for replacement")
    } else {
        let mut result_str = format!(
            "Replaced {} occurrences in {} files for pattern '{}' with '{}' in files matching '{}':\n\n",
            total_replacements, total_files_changed, params.pattern, params.replacement, params.glob_pattern
        );

        for result in &replacement_results {
            result_str.push_str(&format!(
                "{}: {} replacements\n",
                result.path, result.matches
            ));
        }

        if skipped_files > 0 {
            result_str.push_str(&format!(
                "\n\nSkipped {} files due to permissions or read/write errors",
                skipped_files
            ));
        }

        json!(result_str)
    }
}
