use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Project {
    cwd: PathBuf,
}

impl Project {
    pub fn new(cwd: PathBuf) -> Self {
        Self { cwd }
    }

    pub fn get_cwd(&self) -> PathBuf {
        self.cwd.clone()
    }
}