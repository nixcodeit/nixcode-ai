use core::str;
use std::sync::Arc;

use git2::{Status, SubmoduleIgnore};
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::utils::resolve_repository;
use crate::project::Project;

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

            let wstatus = match status {
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
