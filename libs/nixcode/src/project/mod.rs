use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Project {
    cwd: PathBuf,
    project_init_analysis_content: Option<String>,
}

impl Project {
    pub fn new(cwd: PathBuf) -> Self {
        let init_analysis_path = cwd.join(".nixcode/init.md");
        let mut project_init_analysis_content = None;
        if let Ok(_) = std::fs::exists(init_analysis_path.as_path()) {
            project_init_analysis_content = Some(std::fs::read_to_string(init_analysis_path).unwrap());
        }
        Self { cwd, project_init_analysis_content }
    }

    pub fn get_cwd(&self) -> PathBuf {
        self.cwd.clone()
    }

    pub fn get_project_init_analysis_content(&self) -> Option<String> {
        self.project_init_analysis_content.clone()
    }

    pub fn has_init_analysis(&self) -> bool {
        self.project_init_analysis_content.is_some()
    }
}