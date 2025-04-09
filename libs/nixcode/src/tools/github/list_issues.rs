use crate::project::Project;
use crate::tools::github::validate_repo_params;
use nixcode_macros::tool;
use octocrab::params;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(JsonSchema, Serialize, Deserialize, Debug)]
pub struct GithubIssuesListParams {
    #[schemars(
        description = "Github account/organization name (default read from config, format: <org>/<repo>)"
    )]
    pub org: Option<String>,

    #[schemars(
        description = "Github repository name (default read from config, format: <org>/<repo>)"
    )]
    pub repo: Option<String>,
}

#[tool("Get list of open issues from GitHub")]
pub async fn github_issues_list(
    params: GithubIssuesListParams,
    project: Arc<Project>,
) -> serde_json::Value {
    log::debug!("github_issues_list({:?})", params);
    let (org, repo) = match validate_repo_params(&params.org, &params.repo, project) {
        Ok(value) => value,
        Err(value) => return value,
    };
    let client = octocrab::instance();
    let issues = client
        .issues(org, repo)
        .list()
        .state(params::State::Open)
        .per_page(100)
        .send()
        .await;

    match issues {
        Ok(issues) => {
            let mut result = String::new();
            for issue in issues.items {
                result.push_str(format!("#{}: {}\n", issue.number, issue.title).as_str());
            }
            serde_json::to_value(result).unwrap()
        }
        Err(e) => json!(format!("Error fetching issues: {}", e)),
    }
}
