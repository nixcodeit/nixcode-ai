use std::sync::Arc;

use crate::project::Project;
use crate::tools::fs::file_partial_utils;
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct UpdateTextFilePartialParams {
    #[schemars(description = "Relative path to file")]
    pub path: String,

    #[schemars(description = "New content for the specified part of the file")]
    pub content: String,

    #[schemars(description = "Starting line number for partial update (0-based, inclusive)")]
    pub start_line: Option<usize>,

    #[schemars(description = "Ending line number for partial update (0-based, inclusive)")]
    pub end_line: Option<usize>,

    #[schemars(description = "Character range for partial update in format 'start:end'")]
    pub range: Option<String>,
}

#[tool(
    "Update part of a file content, keep in mind after updating the file, line numbers will change"
)]
pub async fn update_text_file_partial(
    params: UpdateTextFilePartialParams,
    project: Arc<Project>,
) -> serde_json::Value {
    // Validate and resolve file path
    let path = match file_partial_utils::validate_file_path(&params.path, project) {
        Ok(p) => p,
        Err(e) => return e,
    };

    // Read the current file content
    let current_content = match file_partial_utils::read_file_content(&path).await {
        Ok(content) => content,
        Err(e) => return json!(e),
    };

    // Update the content based on the provided parameters
    let updated_content = match generate_updated_content(&params, &current_content) {
        Ok(content) => content,
        Err(e) => return json!(e),
    };

    // Write the updated content back to the file
    match file_partial_utils::write_file_content(&path, &updated_content, "updated").await {
        Ok(message) => json!(message),
        Err(e) => json!(e),
    }
}

// Helper function to generate updated content based on parameters
fn generate_updated_content(
    params: &UpdateTextFilePartialParams,
    current_content: &str,
) -> Result<String, String> {
    // Case 1: Line-based update with both start and end line
    if let (Some(start_line), Some(end_line)) = (params.start_line, params.end_line) {
        return file_partial_utils::update_by_line_range(
            current_content,
            start_line,
            end_line,
            Some(&params.content),
        );
    }

    // Case 2: Character range update
    if let Some(range_str) = &params.range {
        return file_partial_utils::handle_char_range(
            current_content,
            range_str,
            Some(&params.content),
        );
    }

    // Case 3: Single line update (only start_line provided)
    if let Some(start_line) = params.start_line {
        return file_partial_utils::update_by_line_range(
            current_content,
            start_line,
            start_line,
            Some(&params.content),
        );
    }

    // No update method specified
    Err(
        "No partial update method specified. Please provide start_line, end_line, or range."
            .to_string(),
    )
}
