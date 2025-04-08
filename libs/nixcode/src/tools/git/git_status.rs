use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitGetTreeProps {}

#[tool("Get git status")]
pub async fn git_status(_: GitGetTreeProps, project: Arc<Project>) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());
    
    let mut cmd = Command::new("git");
    cmd.current_dir(current_dir)
       .arg("status")
       .arg("--porcelain")
       .arg("-z");
    
    let output = run_git_command(cmd).await;
    
    // If the output is empty or just whitespace, the working tree is clean
    if let Some(result) = output.as_str() {
        if result.trim().is_empty() {
            return serde_json::json!("Working tree clean");
        }
        
        // Process the porcelain output to match the previous format
        let mut formatted_output = String::new();
        
        // Split by null character (0 byte) which is used by git status -z
        for entry in result.split('\0') {
            if entry.is_empty() {
                continue;
            }
            
            // First two characters are the status codes
            if entry.len() >= 2 {
                let status_code = &entry[0..2];
                let file_path = &entry[3..];
                
                // Convert the status code to the format used by the previous implementation
                let formatted_status = match status_code.trim() {
                    "A " => "A  ",
                    " A" => " A ",
                    "M " => "M  ",
                    " M" => " M ",
                    "D " => "D  ",
                    " D" => " D ",
                    "R " => "R  ",
                    " R" => " R ",
                    "C " => "C  ",
                    " C" => " C ",
                    "??" => "?? ",
                    "!!" => "!! ",
                    _ => status_code,
                };
                
                formatted_output.push_str(&format!("{} {}\n", formatted_status, file_path));
            } else {
                // Just in case there's an unexpected format
                formatted_output.push_str(&format!("{}\n", entry));
            }
        }
        
        return serde_json::json!(formatted_output);
    }
    
    // If we get here, there was likely an error
    output
}