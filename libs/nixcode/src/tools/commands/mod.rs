use serde_json::json;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio_stream::wrappers::LinesStream;
use tokio_stream::StreamExt;

pub mod cargo_build;
pub mod cargo_fix;
pub mod cargo_fmt;
pub mod cargo_test;

async fn run_command(mut command: Command) -> serde_json::Value {
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    if let Err(e) = child {
        return json!(format!("Failed to execute cargo build: {}", e));
    }

    let mut child: Child = child.unwrap();

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let stdout = LinesStream::new(BufReader::new(stdout).lines());
    let stderr = LinesStream::new(BufReader::new(stderr).lines());
    let mut merged = StreamExt::merge(stdout, stderr);

    let mut output = String::new();

    while let Some(line) = merged.next().await {
        if line.is_err() {
            continue;
        }
        output.push_str(format!("{}\n", line.unwrap()).as_str());
    }

    let exit_code = child.wait().await;
    let exit_code = match exit_code {
        Ok(code) => code.to_string(),
        Err(e) => e.to_string(),
    };

    let text_result = format!("{}\n\nExit code: {}", output, exit_code);

    json!(text_result)
}
