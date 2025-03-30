use std::sync::Arc;

use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::utils::resolve_repository;
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
    let repository = resolve_repository(project.get_repo_path());
    if repository.is_none() {
        return json!("Not a git repository");
    }

    let repository = repository.unwrap();
    let pattern = props.pattern.as_deref();
    let sort_by_date = props.sort_by_date.unwrap_or(false);
    let show_messages = props.show_messages.unwrap_or(false);

    // Get all tags
    let tag_names = match repository.tag_names(pattern) {
        Ok(names) => names,
        Err(e) => return json!(format!("Failed to list tags: {}", e)),
    };

    if tag_names.len() == 0 {
        return json!("No tags found");
    }

    // Prepare tag information for sorting and display
    let mut tag_info: Vec<(String, Option<git2::Time>, Option<String>)> = Vec::new();

    for i in 0..tag_names.len() {
        if let Some(name) = tag_names.get(i) {
            let tag_name = name.to_string();
            let mut tag_time = None;
            let mut tag_message = None;

            // Try to get tag object if we need creation date or message
            if sort_by_date || show_messages {
                // First lookup the tag reference
                if let Ok(tag_ref) = repository.find_reference(&format!("refs/tags/{}", tag_name)) {
                    if let Ok(tag_obj) = tag_ref.peel_to_tag() {
                        // For an annotated tag, get timestamp and message
                        if show_messages {
                            tag_message = tag_obj.message().map(|s| s.to_string());
                        }

                        if sort_by_date {
                            if let Some(tagger) = tag_obj.tagger() {
                                tag_time = Some(tagger.when());
                            }
                        }
                    } else if sort_by_date {
                        // For lightweight tags, we can try to get commit time
                        if let Ok(target) = tag_ref.peel_to_commit() {
                            tag_time = Some(target.time());
                        }
                    }
                }
            }

            tag_info.push((tag_name, tag_time, tag_message));
        }
    }

    // Sort tags
    if sort_by_date {
        tag_info.sort_by(|a, b| {
            match (a.1, b.1) {
                (Some(a_time), Some(b_time)) => b_time.seconds().cmp(&a_time.seconds()), // Newest first
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.0.cmp(&b.0), // Fallback to alphabetical
            }
        });
    } else {
        // Sort alphabetically
        tag_info.sort_by(|a, b| a.0.cmp(&b.0));
    }

    // Format result
    let mut result = String::new();

    if tag_info.is_empty() {
        result.push_str("No tags found");
    } else {
        result.push_str("Tags:\n");

        for (name, time, message) in tag_info {
            // Format tag entry
            let date_str = if let Some(time) = time {
                // Convert git timestamp to human readable format
                let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(time.seconds(), 0)
                    .unwrap_or_else(|| chrono::Utc::now());
                format!(" (created: {})", datetime.format("%Y-%m-%d %H:%M:%S"))
            } else {
                String::new()
            };

            result.push_str(&format!("* {}{}\n", name, date_str));

            // Add message if available and requested
            if show_messages {
                if let Some(msg) = message {
                    let formatted_message = msg.trim().lines().collect::<Vec<&str>>().join("\n  ");

                    if !formatted_message.is_empty() {
                        result.push_str(&format!("  {}\n", formatted_message));
                    }
                }
            }
        }
    }

    json!(result)
}
