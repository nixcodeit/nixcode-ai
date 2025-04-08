use crate::project::Project;
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
    let project_github = project.get_github();
    let org = params
        .org
        .or_else(|| project_github.clone().and_then(|p| p.org));
    let repo = params
        .repo
        .or_else(|| project_github.clone().and_then(|p| p.repo));
    if org.is_none() || repo.is_none() {
        return json!("GitHub organization or repository not specified");
    }
    let org = org.unwrap();
    let repo = repo.unwrap();
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
