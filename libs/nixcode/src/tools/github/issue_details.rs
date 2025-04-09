use crate::project::Project;
use crate::tools::github;
use nixcode_macros::tool;
use octocrab::models::IssueState;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(JsonSchema, Serialize, Deserialize, Debug)]
pub struct GithubIssueDetailsParams {
    #[schemars(description = "ID of the issue to fetch details for")]
    pub issue_id: u64,

    #[schemars(
        description = "Github account/organization name (default read from config, format: <org>/<repo>)"
    )]
    pub org: Option<String>,

    #[schemars(
        description = "Github repository name (default read from config, format: <org>/<repo>)"
    )]
    pub repo: Option<String>,
}

#[tool("Get details of issue from GitHub")]
pub async fn github_issue_details(
    params: GithubIssueDetailsParams,
    project: Arc<Project>,
) -> Value {
    log::debug!("get_issue_details({:?})", params);
    let (org, repo) = match github::validate_repo_params(&params.org, &params.repo, project) {
        Ok(value) => value,
        Err(value) => return value,
    };

    let client = octocrab::instance();
    let issue = client.issues(org, repo).get(params.issue_id).await;

    if let Err(e) = issue {
        return json!(format!("Error fetching issue: {}", e));
    }

    let issue = issue.unwrap();

    let state = match issue.state {
        IssueState::Open => "open",
        IssueState::Closed => "closed",
        _ => "unknown",
    };

    let details = format!(
        "Title: {}\nState: {}\nCreated At: {}\nUpdated At: {}\nBody: {}",
        issue.title,
        state,
        issue.created_at,
        issue.updated_at,
        issue.body.unwrap_or_default(),
    );

    serde_json::to_value(details).unwrap()
}
