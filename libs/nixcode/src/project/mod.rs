use git2::Repository;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Project {
    cwd: PathBuf,
    project_init_analysis_content: Option<String>,
    repo_path: Option<PathBuf>,
}

impl Project {
    pub fn new(cwd: PathBuf) -> Self {
        let init_analysis_path = cwd.join(".nixcode/init.md");
        let mut project_init_analysis_content = None;
        if let Ok(exists) = std::fs::exists(init_analysis_path.as_path()) {
            if exists {
                project_init_analysis_content =
                    Some(std::fs::read_to_string(init_analysis_path).unwrap());
            }
        }

        let repository = if let Some(repository) = Repository::discover(cwd.as_path()).ok() {
            repository.workdir().map(|path| path.into())
        } else {
            None
        };

        Self {
            cwd,
            project_init_analysis_content,
            repo_path: repository,
        }
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

    pub fn has_repo_path(&self) -> bool {
        self.repo_path.is_some()
    }

    pub fn get_repo_path(&self) -> Option<PathBuf> {
        self.repo_path.clone()
    }
}
