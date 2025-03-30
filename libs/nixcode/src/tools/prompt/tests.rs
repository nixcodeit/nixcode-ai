use super::*;
use crate::project::Project;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::test]
async fn test_get_project_analysis_prompt() {
    let project = Arc::new(Project::new(PathBuf::from("/tmp")));
    let params = get_project_analysis_prompt::ProjectAnalysisPromptParams { focus: None };

    let result = get_project_analysis_prompt::get_project_analysis_prompt(params, project).await;
    assert!(result.is_string());
    let prompt = result.as_str().unwrap();
    assert!(prompt.contains("Project Analysis Task"));
    assert!(prompt.contains("Save your analysis to `.nixcode/init.md`"));
}

#[tokio::test]
async fn test_get_project_analysis_prompt_with_focus() {
    let project = Arc::new(Project::new(PathBuf::from("/tmp")));
    let params = get_project_analysis_prompt::ProjectAnalysisPromptParams {
        focus: Some("architecture".to_string()),
    };

    let result = get_project_analysis_prompt::get_project_analysis_prompt(params, project).await;
    assert!(result.is_string());
    let prompt = result.as_str().unwrap();
    assert!(prompt.contains("Special Focus Area: architecture"));
}
