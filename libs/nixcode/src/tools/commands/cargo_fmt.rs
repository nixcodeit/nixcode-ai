use std::sync::Arc;

use crate::project::Project;
use crate::tools::commands::run_command;
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct CargoFmtProps {
    #[serde(default)]
    pub check: Option<bool>,
    #[serde(default)]
    pub all: Option<bool>,
    #[serde(default)]
    pub path: Option<String>,
}

#[tool("Format Rust code using cargo fmt")]
pub async fn cargo_fmt(props: CargoFmtProps, project: Arc<Project>) -> serde_json::Value {
    let check = props.check.unwrap_or(false);
    let all = props.all.unwrap_or(false);
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());

    let mut cmd = Command::new("cargo");
    cmd.current_dir(current_dir).arg("fmt").arg("-q");

    if check {
        cmd.arg("--check");
    }

    if all {
        cmd.arg("--all");
    }

    if let Some(path) = props.path {
        cmd.arg("--").arg(path);
    }

    run_command(cmd).await
}
