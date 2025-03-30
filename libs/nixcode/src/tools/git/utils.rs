use git2::Repository;
use std::path::PathBuf;

/// Resolves the repository from a given path
pub fn resolve_repository(path: Option<PathBuf>) -> Option<Repository> {
    let repo_path = path?;
    Repository::open(repo_path).ok()
}
