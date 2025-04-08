use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::utils::run_git_command;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize, Default)]
pub struct GitTagsListProps {
    #[schemars(description = "Filter tags by pattern (supports glob patterns)")]
    pub pattern: Option<String>,

    #[schemars(description = "Sort tags by creation date (default: false - alphabetical)")]
    pub sort_by_date: Option<bool>,

    #[schemars(description = "Show tag messages for annotated tags (default: false)")]
    pub show_messages: Option<bool>,
}

#[tool("List git tags")]
pub async fn git_tags_list(props: GitTagsListProps, project: Arc<Project>) -> serde_json::Value {
    let current_dir = project.get_repo_path().unwrap_or(project.get_cwd());
    let pattern = props.pattern.as_deref();
    let sort_by_date = props.sort_by_date.unwrap_or(false);
    let show_messages = props.show_messages.unwrap_or(false);
    
    let mut cmd = Command::new("git");
    cmd.current_dir(current_dir);
    
    if show_messages {
        // For showing messages, we need to use a different format
        cmd.arg("tag")
           .arg("-n"); // Show annotation message
        
        if sort_by_date {
            // We'll need to do a separate command for sorting by date
            cmd.arg("--sort=-creatordate"); // Sort by date, newest first
        }
    } else {
        // Simple tag listing
        cmd.arg("tag");
        
        if sort_by_date {
            cmd.arg("--sort=-creatordate"); // Sort by date, newest first
        }
    }
    
    // Add pattern if specified
    if let Some(pattern_str) = pattern {
        cmd.arg("--list")
           .arg(pattern_str);
    }
    
    let output = run_git_command(cmd).await;
    
    // Format the output
    if let Some(result) = output.as_str() {
        if result.trim().is_empty() {
            return serde_json::json!("No tags found");
        }
        
        let formatted = format!("Tags:\n{}", result);
        return serde_json::json!(formatted);
    }
    
    output
}