use crate::project::Project;
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct ProjectAnalysisPromptParams {
    #[schemars(description = "Optional focus area for analysis (e.g., 'architecture', 'dependencies', 'workflow')")]
    #[serde(default)]
    focus: Option<String>,
}

#[tool("Generate a project analysis prompt for LLM to understand codebase structure and architecture")]
pub async fn get_project_analysis_prompt(params: ProjectAnalysisPromptParams, _project: &Project) -> serde_json::Value {
    let focus = params.focus.unwrap_or_default();
    
    let mut prompt = String::from(
        "# Project Analysis Task\n\n\
        ## Objective\n\
        Analyze this codebase to create a comprehensive understanding of its structure, architecture, and organization. \
        This analysis will help establish a foundation for more efficient collaboration.\n\n\
        ## Analysis Process\n\
        1. **File Structure Exploration**:\n\
           - Use glob patterns to identify key directories and files (`search_glob_files`)\n\
           - Focus on examining high-level directories first, then dive into specific components\n\
           - Identify important configuration files, entry points, and core modules\n\n\
        2. **Architectural Understanding**:\n\
           - Identify the project's architectural pattern(s)\n\
           - Map dependencies between modules/components\n\
           - Document the responsibility of each major component\n\
           - Note any design patterns implemented\n\n\
        3. **Technology Stack Assessment**:\n\
           - Identify programming languages and frameworks used\n\
           - Document key libraries and dependencies\n\
           - Note any tools or services integrated\n\n\
        4. **Project Conventions**:\n\
           - Observe code organization patterns\n\
           - Identify naming conventions\n\
           - Note file/folder organization patterns\n\n\
        5. **Documentation Review**:\n\
           - Examine any existing documentation\n\
           - Look for comments in key files that explain architecture decisions\n\n");

    if !focus.is_empty() {
        prompt.push_str(&format!("## Special Focus Area: {}\n\
            Pay particular attention to aspects related to '{}' in your analysis.\n\n", focus, focus));
    }

    prompt.push_str(
        "## Output Format\n\
        Create a well-structured Markdown document with the following sections:\n\n\
        1. **Project Overview**: High-level description of what the project does\n\
        2. **Architecture**: Detailed explanation of the architectural approach\n\
        3. **Key Components**: Description of main modules/components and their responsibilities\n\
        4. **Workflow**: How data/control flows through the system\n\
        5. **Technology Stack**: Languages, frameworks, and major libraries\n\
        6. **Organization Patterns**: File structure, naming conventions, and organization\n\
        7. **Recommendations**: Suggestions for better understanding specific parts of the codebase\n\n\
        ## Important Instructions\n\
        - Be thorough but concise\n\
        - Use diagrams (described in text) where helpful\n\
        - Include file paths when discussing specific components\n\
        - Focus on understanding the 'why' behind architectural decisions\n\
        - Write such an analysis that will be useful in the system prompt for LLM's future tasks\n\
        - Save your analysis to `.nixcode/init.md`\n\n\
        Begin by exploring the top-level directories and key files to get a comprehensive overview of the project structure."
    );

    serde_json::json!(prompt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Project;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_get_project_analysis_prompt() {
        let project = Project::new(PathBuf::from("/tmp"));
        let params = ProjectAnalysisPromptParams {
            focus: None,
        };

        let result = get_project_analysis_prompt(params, &project).await;
        assert!(result.is_string());
        let prompt = result.as_str().unwrap();
        assert!(prompt.contains("Project Analysis Task"));
        assert!(prompt.contains("Save your analysis to `.nixcode/init.md`"));
    }

    #[tokio::test]
    async fn test_get_project_analysis_prompt_with_focus() {
        let project = Project::new(PathBuf::from("/tmp"));
        let params = ProjectAnalysisPromptParams {
            focus: Some("architecture".to_string()),
        };

        let result = get_project_analysis_prompt(params, &project).await;
        assert!(result.is_string());
        let prompt = result.as_str().unwrap();
        assert!(prompt.contains("Special Focus Area: architecture"));
    }
}