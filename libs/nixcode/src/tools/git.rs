use core::str;
use std::{any::Any, borrow::BorrowMut, path::PathBuf, sync::Arc};

use git2::{DiffOptions, Status, SubmoduleIgnore};
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitAddParams {
    #[schemars(description = "Array of files that will be added to index")]
    files: Vec<String>,
}

fn resolve_repository(path: Option<PathBuf>) -> Option<git2::Repository> {
    let repo_path = path?;

    git2::Repository::open(repo_path).ok()
}

#[tool("Track changes in git")]
pub async fn git_add(props: GitAddParams, project: Arc<Project>) -> serde_json::Value {
    let repository = resolve_repository(project.get_repo_path());
    if repository.is_none() {
        return json!("Not a git repository");
    }

    let repository = repository.unwrap();

    let index = repository.index();

    if let Err(e) = index {
        return json!(e.to_string());
    }

    let mut index = index.unwrap();
    let mut result = String::new();

    for file_path in props.files {
        let path = PathBuf::from(file_path.clone());
        if let Err(e) = index.add_path(path.as_path()) {
            result.push_str(format!("Cannot add {}, reason: {}\n", file_path, e).as_str());
        } else {
            result.push_str(format!("Added {}\n", file_path).as_str());
        }
    }

    let write_result = index.write();
    if let Err(e) = write_result {
        return json!(format!("Cannot save index, reason: {}", e));
    }

    serde_json::to_value(result).unwrap()
}

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitCommitProps {
    #[schemars(description = "Message for commit")]
    message: String,
}

#[tool("Commit changes")]
pub async fn git_commit(props: GitCommitProps, project: Arc<Project>) -> serde_json::Value {
    let repository = resolve_repository(project.get_repo_path());
    if repository.is_none() {
        return json!("Not a git repository");
    }

    let repo = repository.unwrap();

    let index = repo.index();
    if let Err(e) = index {
        return json!(format!("Cannot get index, reason: {}", e));
    }
    let mut index = index.unwrap();
    let oid = index.write_tree();
    if let Err(e) = oid {
        return json!(format!("Cannot write index tree, reason: {}", e));
    }
    let oid = oid.unwrap();

    let signature = repo.signature();
    if let Err(e) = signature {
        return json!(format!("Cannot get signature for commiter, reason: {}", e));
    }
    let signature = signature.unwrap();
    let tree = repo.find_tree(oid);
    if let Err(e) = tree {
        return json!(format!("Cannot find tree for oid: {}, reason: {}", oid, e));
    }
    let tree = tree.unwrap();

    let parent_commit = repo.head();
    if let Err(e) = parent_commit {
        return json!(format!("Cannot get HEAD ref, reason: {}", e));
    }

    let parent_commit = parent_commit.unwrap().peel_to_commit();
    if let Err(e) = parent_commit {
        return json!(format!("Cannot peel to commit, reason: {}", e));
    }
    let parent_commit = parent_commit.unwrap();

    let result = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        props.message.as_str(),
        &tree,
        &[&parent_commit],
    );

    if let Err(e) = result {
        return json!(format!("Can't commit, reason: {}", e));
    }

    json!("Commit created")
}

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitGetTreeProps {}

#[tool("Get git status")]
pub async fn git_status(_: GitGetTreeProps, project: Arc<Project>) -> serde_json::Value {
    let repository = resolve_repository(project.get_repo_path());
    if repository.is_none() {
        return json!("Not a git repository");
    }

    let repository = repository.unwrap();

    let statuses = repository.statuses(None);
    if let Err(e) = statuses {
        return json!(format!("Cannot get statuses, reason: {}", e));
    }
    let statuses = statuses.unwrap();

    if statuses.is_empty() {
        return json!("Working tree clean");
    }

    let mut result = String::new();

    statuses
        .into_iter()
        .filter(|e| e.status() != Status::CURRENT)
        .for_each(|entry| {
            let status = entry.status();
            let mut istatus = match status {
                Status::INDEX_NEW => 'A',
                Status::INDEX_MODIFIED => 'M',
                Status::INDEX_DELETED => 'D',
                Status::INDEX_RENAMED => 'R',
                Status::INDEX_TYPECHANGE => 'T',
                _ => ' ',
            };

            let mut wstatus = match status {
                s if s.contains(git2::Status::WT_NEW) => {
                    if istatus == ' ' {
                        istatus = '?';
                    }
                    '?'
                }
                s if s.contains(git2::Status::WT_MODIFIED) => 'M',
                s if s.contains(git2::Status::WT_DELETED) => 'D',
                s if s.contains(git2::Status::WT_RENAMED) => 'R',
                s if s.contains(git2::Status::WT_TYPECHANGE) => 'T',
                _ => ' ',
            };

            if status.contains(Status::IGNORED) {
                return;
            }

            let mut extra = "";
            let status = entry.index_to_workdir().and_then(|diff| {
                let ignore = SubmoduleIgnore::Unspecified;
                diff.new_file()
                    .path_bytes()
                    .and_then(|s| str::from_utf8(s).ok())
                    .and_then(|name| repository.submodule_status(name, ignore).ok())
            });
            if let Some(status) = status {
                if status.contains(git2::SubmoduleStatus::WD_MODIFIED) {
                    extra = " (new commits)";
                } else if status.contains(git2::SubmoduleStatus::WD_INDEX_MODIFIED)
                    || status.contains(git2::SubmoduleStatus::WD_WD_MODIFIED)
                {
                    extra = " (modified content)";
                } else if status.contains(git2::SubmoduleStatus::WD_UNTRACKED) {
                    extra = " (untracked content)";
                }
            }

            let (mut a, mut b, mut c) = (None, None, None);
            if let Some(diff) = entry.head_to_index() {
                a = diff.old_file().path();
                b = diff.new_file().path();
            }
            if let Some(diff) = entry.index_to_workdir() {
                a = a.or_else(|| diff.old_file().path());
                b = b.or_else(|| diff.old_file().path());
                c = diff.new_file().path();
            }

            let file_status = match (istatus, wstatus) {
                ('R', 'R') => format!(
                    "RR {} {} {}{}",
                    a.unwrap().display(),
                    b.unwrap().display(),
                    c.unwrap().display(),
                    extra
                ),
                ('R', w) => format!(
                    "R{} {} {}{}",
                    w,
                    a.unwrap().display(),
                    b.unwrap().display(),
                    extra
                ),
                (i, 'R') => format!(
                    "{}R {} {}{}",
                    i,
                    a.unwrap().display(),
                    c.unwrap().display(),
                    extra
                ),
                (i, w) => format!("{}{} {}{}", i, w, a.unwrap().display(), extra),
            };

            result.push_str(format!("{}\n", file_status).as_str());
        });

    json!(result)
}

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitDiffProps {
    #[schemars(description = "Path to the file to show diff for")]
    file_path: String,
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

    let mut diff_result = String::new();

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

        diff_result = diff_output;

        // If empty, the file might be staged
        if diff_result.is_empty() {
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

            diff_result = staged_output;
        }
    } else {
        // No HEAD yet, show the entire file as new
        match std::fs::read_to_string(&full_path) {
            Ok(content) => {
                diff_result = format!("New file: {}\n\n{}", props.file_path, content);
            }
            Err(e) => {
                return json!(format!("Error reading file: {}", e));
            }
        }
    }

    if diff_result.is_empty() {
        return json!(format!("No changes detected for file: {}", props.file_path));
    }

    json!(diff_result)
}
