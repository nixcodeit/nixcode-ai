use std::sync::Arc;

use crate::project::Project;
use crate::tools::commands::run_command;
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct CargoTestProps {
    #[serde(default)]
    pub package: Option<String>,
    #[serde(default)]
    pub test: Option<String>,
    #[serde(default)]
    pub lib: Option<bool>,
    #[serde(default)]
    pub verbose: Option<bool>,
    #[serde(default)]
    pub quiet: Option<bool>,
    #[serde(default)]
    pub nocapture: Option<bool>,
}

#[tool("Run tests for a Rust project using cargo test")]
pub async fn cargo_test(props: CargoTestProps, project: Arc<Project>) -> serde_json::Value {
    let lib = props.lib.unwrap_or(false);
    let verbose = props.verbose.unwrap_or(false);
    let quiet = props.quiet.unwrap_or(false);
    let nocapture = props.nocapture.unwrap_or(false);
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());

    let mut cmd = Command::new("cargo");
    cmd.current_dir(current_dir).arg("test").arg("-q");

    if let Some(package) = props.package {
        cmd.arg("--package").arg(package);
    }

    if lib {
        cmd.arg("--lib");
    }

    if verbose {
        cmd.arg("--verbose");
    }

    if quiet {
        cmd.arg("--quiet");
    }

    if let Some(test) = props.test {
        cmd.arg(test);
    }

    if nocapture {
        cmd.arg("--").arg("--nocapture");
    }

    run_command(cmd).await
}
