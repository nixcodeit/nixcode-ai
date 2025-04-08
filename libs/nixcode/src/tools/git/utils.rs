use serde_json::json;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio_stream::wrappers::LinesStream;
use tokio_stream::StreamExt;

/// Run a git command and return the output as a JSON value
pub async fn run_git_command(mut command: Command) -> serde_json::Value {
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    if let Err(e) = child {
        return json!(format!("Failed to execute git command: {}", e));
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
    
    if let Ok(status) = exit_code {
        if status.success() {
            // If the command was successful, just return the output
            return json!(output.trim());
        }
    }
    
    // If there was an error, include the exit code
    let exit_code = match exit_code {
        Ok(code) => code.to_string(),
        Err(e) => e.to_string(),
    };

    let text_result = format!("{}\n\nExit code: {}", output, exit_code);
    json!(text_result)
}

/// Check if the current directory is a git repository
pub async fn is_git_repository(path: Option<PathBuf>) -> bool {
    if let Some(repo_path) = path {
        let mut cmd = Command::new("git");
        cmd.current_dir(repo_path).arg("rev-parse").arg("--is-inside-work-tree");
        
        let output = run_git_command(cmd).await;
        if let Some(result) = output.as_str() {
            return result.trim() == "true";
        }
    }
    false
}