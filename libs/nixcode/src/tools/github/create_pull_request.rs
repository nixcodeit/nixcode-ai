use crate::project::Project;
use crate::tools::github;
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(JsonSchema, Serialize, Deserialize, Debug)]
pub struct CreatePullRequestParams {
    #[schemars(description = "Title of the pull request")]
    pub title: String,

    #[schemars(description = "Body of the pull request")]
    pub body: String,

    #[schemars(description = "Branch to merge into")]
    pub base: String,

    #[schemars(description = "Branch to merge from")]
    pub head: String,

    #[schemars(
        description = "Github account/organization name (default read from config, format: <org>/<repo>)"
    )]
    pub org: Option<String>,

    #[schemars(
        description = "Github repository name (default read from config, format: <org>/<repo>)"
    )]
    pub repo: Option<String>,
}

#[tool("Create a pull request on GitHub")]
pub async fn github_create_pull_request(
    params: CreatePullRequestParams,
    project: Arc<Project>,
) -> Value {
    let (org, repo) = match github::validate_repo_params(&params.org, &params.repo, project) {
        Ok(value) => value,
        Err(value) => return value,
    };

    let client = octocrab::instance();

    let pull_request = client
        .pulls(org.clone(), repo.clone())
        .create(params.title, params.head, params.base)
        .body(params.body)
        .send()
        .await;

    if let Err(e) = pull_request {
        return json!(format!("Error creating pull request: {}", e));
    }

    let pull_request = pull_request.unwrap();
    let pull_request_number = pull_request.number;
    let url = pull_request.html_url;
    let url = match url {
        Some(url) => url.to_string(),
        None => format!(
            "https://github.com/{}/{}/pull/{}",
            org, repo, pull_request_number
        ),
    };

    json!(format!(
        "Pull request #{} created successfully: {}",
        pull_request_number, url
    ))
}
