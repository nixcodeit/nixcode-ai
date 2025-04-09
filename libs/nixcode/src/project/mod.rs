use crate::config::GitHubSettings;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct GitHub {
    /// GitHub account/organization name
    pub org: Option<String>,

    /// GitHub repository name
    pub repo: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Project {
    cwd: PathBuf,
    project_init_analysis_content: Option<String>,
    repo_path: Option<PathBuf>,
    github: Option<GitHub>,
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

        // Try to find git repository root using git rev-parse
        let repository = {
            let output = Command::new("git")
                .current_dir(&cwd)
                .args(["rev-parse", "--show-toplevel"])
                .output()
                .ok();

            if let Some(output) = output {
                if output.status.success() {
                    if let Ok(path) = String::from_utf8(output.stdout) {
                        Some(PathBuf::from(path.trim()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        Self {
            cwd,
            project_init_analysis_content,
            repo_path: repository,
            github: None,
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

    pub fn set_github(&mut self, github: &GitHubSettings) -> &mut Self {
        self.github = Some(GitHub {
            org: github.org.clone(),
            repo: github.repo.clone(),
        });

        self
    }

    pub fn get_github(&self) -> Option<GitHub> {
        self.github.clone()
    }
}
