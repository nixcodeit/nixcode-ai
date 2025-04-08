use core::str;
use std::path::PathBuf;
use std::sync::Arc;

use git2::DiffOptions;
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::utils::resolve_repository;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitDiffProps {
    #[schemars(description = "Path to the file to show diff for")]
    pub file_path: String,
}

#[tool("Get file diff")]
pub async fn git_diff(props: GitDiffProps, project: Arc<Project>) -> serde_json::Value {
    let repository = resolve_repository(project.get_repo_path());
    if repository.is_none() {
        return json!("Not a git repository");
    }

    let repo = repository.unwrap();
    let file_path = PathBuf::from(&props.file_path);

    // Check if file exists
    let full_path = project.get_repo_path().unwrap().join(&file_path);
    if !full_path.exists() {
        return json!(format!("File not found: {}", props.file_path));
    }

    // Get the diff
    let mut diff_options = DiffOptions::new();
    diff_options.pathspec(&props.file_path);
    diff_options.context_lines(3);
    diff_options.show_binary(true);

    // Get HEAD tree
    let head_tree = match repo.head() {
        Ok(head) => match head.peel_to_tree() {
            Ok(tree) => Some(tree),
            Err(_) => None, // New repository with no commits
        },
        Err(_) => None, // No HEAD yet
    };

    // If we have a HEAD, compare with it
    if let Some(head_tree) = head_tree {
        let diff =
            match repo.diff_tree_to_workdir_with_index(Some(&head_tree), Some(&mut diff_options)) {
                Ok(diff) => diff,
                Err(e) => return json!(format!("Error creating diff: {}", e)),
            };

        // Format diff output
        let mut diff_output = String::new();
        if let Err(e) = diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let origin = line.origin();
            let content = match str::from_utf8(line.content()) {
                Ok(content) => content,
                Err(_) => return false,
            };

            // Format the output
            let prefix = match origin {
                '+' => "+", // Added
                '-' => "-", // Removed
                'H' => "",  // Hunk header
                'B' => "",  // Binary content
                _ => "",    // Context and other lines
            };

            diff_output.push_str(&format!("{}{}", prefix, content));
            true
        }) {
            return json!(format!("Error printing diff: {}", e));
        }

        // If empty, the file might be staged
        if diff_output.is_empty() {
            let diff = match repo.diff_index_to_workdir(None, Some(&mut diff_options)) {
                Ok(diff) => diff,
                Err(e) => return json!(format!("Error creating diff: {}", e)),
            };

            let mut staged_output = String::new();
            if let Err(e) = diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
                let origin = line.origin();
                let content = match str::from_utf8(line.content()) {
                    Ok(content) => content,
                    Err(_) => return false,
                };

                // Format the output
                let prefix = match origin {
                    '+' => "+", // Added
                    '-' => "-", // Removed
                    'H' => "",  // Hunk header
                    'B' => "",  // Binary content
                    _ => "",    // Context and other lines
                };

                staged_output.push_str(&format!("{}{}", prefix, content));
                true
            }) {
                return json!(format!("Error printing diff: {}", e));
            }

            if !staged_output.is_empty() {
                return json!(staged_output);
            }
        } else {
            return json!(diff_output);
        }
    } else {
        // No HEAD yet, show the entire file as new
        match std::fs::read_to_string(&full_path) {
            Ok(content) => {
                return json!(format!("New file: {}\n\n{}", props.file_path, content));
            }
            Err(e) => {
                return json!(format!("Error reading file: {}", e));
            }
        }
    }

    json!(format!("No changes detected for file: {}", props.file_path))
}