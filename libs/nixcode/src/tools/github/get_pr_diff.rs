use crate::project::Project;
use crate::tools::github;
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(JsonSchema, Serialize, Deserialize, Debug)]
pub struct GetPrDiffParams {
    #[schemars(description = "ID of the pull request")]
    pub id: u64,

    #[schemars(
        description = "Github account/organization name (default read from config, format: <org>/<repo>)"
    )]
    pub org: Option<String>,

    #[schemars(
        description = "Github repository name (default read from config, format: <org>/<repo>)"
    )]
    pub repo: Option<String>,
}

#[tool("Get diff of a pull request from GitHub")]
pub async fn github_pr_diff(params: GetPrDiffParams, project: Arc<Project>) -> Value {
    let (org, repo) = match github::validate_repo_params(&params.org, &params.repo, project) {
        Ok(value) => value,
        Err(value) => return value,
    };

    let client = octocrab::instance();

    match client
        .pulls(org.clone(), repo.clone())
        .get_diff(params.id)
        .await
    {
        Ok(diff) => json!(diff),
        Err(e) => json!(format!("Error fetching pull request diff: {}", e)),
    }
}
