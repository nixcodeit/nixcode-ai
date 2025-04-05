use std::sync::Arc;

use crate::project::Project;
use crate::tools::commands::run_command;
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct CargoFixProps {
    #[serde(default)]
    pub broken_code: Option<bool>,
    #[serde(default)]
    pub edition: Option<bool>,
    #[serde(default)]
    pub package: Option<String>,
}

#[tool("Automatically fix Rust code with cargo fix")]
pub async fn cargo_fix(props: CargoFixProps, project: Arc<Project>) -> serde_json::Value {
    let broken_code = props.broken_code.unwrap_or(false);
    let edition = props.edition.unwrap_or(false);
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());

    let mut cmd = Command::new("cargo");
    cmd.current_dir(current_dir)
        .arg("fix")
        .arg("-q")
        .arg("--allow-dirty")
        .arg("--allow-staged");

    if broken_code {
        cmd.arg("--broken-code");
    }

    if edition {
        cmd.arg("--edition");
    }

    if let Some(package) = props.package {
        cmd.arg("--package").arg(package);
    }

    run_command(cmd).await
}
