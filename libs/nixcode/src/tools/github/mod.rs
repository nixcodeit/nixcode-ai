use crate::project::Project;
use serde_json::{json, Value};
use std::sync::Arc;

pub mod add_comment_to_pull_request;
pub mod create_pull_request;
mod get_pr_diff;
pub mod issue_details;
pub mod list_issues;

pub fn validate_repo_params(
    org: &Option<String>,
    repo: &Option<String>,
    project: Arc<Project>,
) -> Result<(String, String), Value> {
    let project_github = project.get_github();
    let org = org
        .clone()
        .or_else(|| project_github.clone().and_then(|p| p.org));
    let repo = repo
        .clone()
        .or_else(|| project_github.clone().and_then(|p| p.repo));
    if org.is_none() || repo.is_none() {
        return Err(json!("GitHub organization or repository not specified"));
    }
    Ok((org.unwrap(), repo.unwrap()))
}
