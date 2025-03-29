use core::str;
use std::{any::Any, borrow::BorrowMut, path::PathBuf, sync::Arc};

use git2::{Status, SubmoduleIgnore};
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
