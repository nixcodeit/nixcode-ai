use std::path::PathBuf;
use std::sync::Arc;

use crate::project::Project;
use serde_json::json;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Helper function to read file content
pub async fn read_file_content(path: &PathBuf) -> Result<String, String> {
    let mut file = match File::open(path).await {
        Ok(f) => f,
        Err(e) => return Err(format!("Failed to open file: {}", e)),
    };

    let mut content = String::new();
    if let Err(e) = file.read_to_string(&mut content).await {
        return Err(format!("Failed to read file: {}", e));
    }

    Ok(content)
}

/// Helper function to write file content
pub async fn write_file_content(
    path: &PathBuf,
    content: &str,
    operation_type: &str,
) -> Result<String, String> {
    let mut file = match OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .await
    {
        Ok(f) => f,
        Err(e) => return Err(format!("Failed to open file for writing: {}", e)),
    };

    match file.write_all(content.as_bytes()).await {
        Ok(_) => Ok(format!("File partially {}", operation_type)),
        Err(e) => Err(format!("Failed to write to file: {}", e)),
    }
}

/// Helper function to validate and get file path
pub fn validate_file_path(
    file_path: &str,
    project: Arc<Project>,
) -> Result<PathBuf, serde_json::Value> {
    use crate::utils::fs;

    // Validate and resolve file path
    let file_path = PathBuf::from(file_path);
    let cwd = project.get_cwd();
    let path = match fs::join_path(cwd.clone(), file_path) {
        Ok(p) => p,
        Err(e) => return Err(json!(e.to_string())),
    };

    // Ensure path is within project directory
    if !path.starts_with(cwd) {
        return Err(json!("Path must be inside project directory"));
    }

    Ok(path)
}

/// Helper function to update content by line range
pub fn update_by_line_range(
    current_content: &str,
    start_line: usize,
    end_line: usize,
    new_content: Option<&str>,
) -> Result<String, String> {
    let lines: Vec<&str> = current_content.lines().collect();

    // Validate line numbers
    if start_line >= lines.len() || end_line >= lines.len() || start_line > end_line {
        return Err("Invalid line range specified".to_string());
    }

    let mut updated_lines = lines.clone();

    match new_content {
        // Replace mode: replace specified lines with new content
        Some(content) => {
            let new_lines: Vec<&str> = content.lines().collect();
            updated_lines.splice(start_line..=end_line, new_lines.iter().copied());
        }
        // Delete mode: remove specified lines
        None => {
            updated_lines.splice(start_line..=end_line, std::iter::empty());
        }
    }

    Ok(updated_lines.join("\n"))
}

/// Helper function to handle character range operations
pub fn handle_char_range(
    current_content: &str,
    range_str: &str,
    new_content: Option<&str>,
) -> Result<String, String> {
    let range_parts: Vec<&str> = range_str.split(':').collect();
    if range_parts.len() != 2 {
        return Err("Invalid range format. Expected 'start:end'".to_string());
    }

    let start_char = match range_parts[0].parse::<usize>() {
        Ok(n) => n,
        Err(_) => return Err("Invalid start position in range".to_string()),
    };

    let end_char = match range_parts[1].parse::<usize>() {
        Ok(n) => n,
        Err(_) => return Err("Invalid end position in range".to_string()),
    };

    if start_char >= current_content.len()
        || end_char > current_content.len()
        || start_char > end_char
    {
        return Err("Invalid character range specified".to_string());
    }

    // Create the updated content
    match new_content {
        // Replace mode: replace specified range with new content
        Some(content) => Ok(format!(
            "{}{}{}",
            &current_content[..start_char],
            content,
            &current_content[end_char..]
        )),
        // Delete mode: remove specified range
        None => Ok(format!(
            "{}{}",
            &current_content[..start_char],
            &current_content[end_char..]
        )),
    }
}
