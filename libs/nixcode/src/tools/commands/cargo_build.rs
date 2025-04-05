use crate::project::Project;
use crate::tools::commands::run_command;
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct CargoBuildProps {
    #[serde(default)]
    pub release: Option<bool>,
    #[serde(default)]
    pub package: Option<String>,
}

#[tool("Build the Rust project using cargo")]
pub async fn cargo_build(props: CargoBuildProps, project: Arc<Project>) -> serde_json::Value {
    let release = props.release.unwrap_or(false);
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());

    let mut cmd = Command::new("cargo");
    cmd.current_dir(current_dir).arg("build").arg("-q");

    if release {
        cmd.arg("--release");
    }

    if let Some(package) = props.package {
        cmd.arg("--package").arg(package);
    }

    run_command(cmd).await
}
