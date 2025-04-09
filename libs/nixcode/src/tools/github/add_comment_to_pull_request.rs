use crate::project::Project;
use crate::tools::github;
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(JsonSchema, Serialize, Deserialize, Debug)]
pub struct AddCommentToPullRequestParams {
    #[schemars(description = "ID of the pull request")]
    pub id: u64,

    #[schemars(description = "Body of the comment")]
    pub body: String,

    #[schemars(
        description = "Github account/organization name (default read from config, format: <org>/<repo>)"
    )]
    pub org: Option<String>,

    #[schemars(
        description = "Github repository name (default read from config, format: <org>/<repo>)"
    )]
    pub repo: Option<String>,
}

#[tool("Add review to pull request on GitHub")]
pub async fn github_add_pull_request_comment(
    params: AddCommentToPullRequestParams,
    project: Arc<Project>,
) -> Value {
    let (org, repo) = match github::validate_repo_params(&params.org, &params.repo, project) {
        Ok(value) => value,
        Err(value) => return value,
    };

    let client = octocrab::instance();

    if let Err(e) = client.pulls(org.clone(), repo.clone()).get(params.id).await {
        return json!(format!("Error fetching pull request: {}", e));
    }

    match client
        .issues(org, repo)
        .create_comment(params.id, params.body)
        .await
    {
        Ok(comment) => json!(format!("Comment added: {}", comment.html_url)),
        Err(e) => json!(format!("Failed to add comment: {}", e)),
    }
}
