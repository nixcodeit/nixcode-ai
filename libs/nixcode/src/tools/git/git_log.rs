use std::sync::Arc;

use git2::{Oid, Repository, Revwalk};
use nixcode_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::utils::resolve_repository;
use crate::project::Project;

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct GitLogProps {
    #[schemars(description = "Starting reference (commit hash, branch name, or tag)")]
    pub from_ref: Option<String>,
    
    #[schemars(description = "Ending reference (commit hash, branch name, or tag)")]
    pub to_ref: Option<String>,
    
    #[schemars(description = "Maximum number of commits to retrieve")]
    pub limit: Option<usize>,
    
    #[schemars(description = "Path to limit the history to a specific file or directory")]
    pub path: Option<String>,
}

/// Resolves a reference string to an Oid
fn resolve_ref(repo: &Repository, reference: &str) -> Result<Oid, git2::Error> {
    // Try as a reference name first
    if let Ok(reference) = repo.find_reference(reference) {
        if let Ok(commit) = reference.peel_to_commit() {
            return Ok(commit.id());
        }
    }
    
    // Try as a branch name
    if let Ok(branch) = repo.find_branch(reference, git2::BranchType::Local) {
        if let Ok(commit) = branch.get().peel_to_commit() {
            return Ok(commit.id());
        }
    }
    
    // Try as a tag
    if let Ok(tag_names) = repo.tag_names(Some(reference)) {
        for tag_name in tag_names.iter().flatten() {
            if let Ok(tag) = repo.find_reference(&format!("refs/tags/{}", tag_name)) {
                if let Ok(commit) = tag.peel_to_commit() {
                    return Ok(commit.id());
                }
            }
        }
    }
    
    // Try as a direct OID (commit hash)
    Oid::from_str(reference)
}

/// Configure the revwalk to include the appropriate range of commits
fn configure_revwalk(
    repo: &Repository, 
    revwalk: &mut Revwalk, 
    from_ref: Option<&str>, 
    to_ref: Option<&str>
) -> Result<(), git2::Error> {
    // Set up the revision walk based on from_ref and to_ref
    if let Some(to) = to_ref {
        // Resolve the to_ref
        let to_oid = resolve_ref(repo, to)?;
        revwalk.push(to_oid)?;
    } else {
        // If no to_ref is specified, start from HEAD
        revwalk.push_head()?;
    }
    
    // If from_ref is specified, hide all commits reachable from it
    if let Some(from) = from_ref {
        let from_oid = resolve_ref(repo, from)?;
        revwalk.hide(from_oid)?;
    }
    
    Ok(())
}

#[tool("Get git log between refs")]
pub async fn git_log(props: GitLogProps, project: Arc<Project>) -> serde_json::Value {
    let repository = match resolve_repository(project.get_repo_path()) {
        Some(repo) => repo,
        None => return json!("Not a git repository"),
    };
    
    let limit = props.limit.unwrap_or(50); // Default to 50 commits
    
    // Create a revwalk (iterator over commits)
    let mut revwalk = match repository.revwalk() {
        Ok(revwalk) => revwalk,
        Err(e) => return json!(format!("Failed to create revision walker: {}", e)),
    };
    
    // Sort by time (most recent first)
    if let Err(e) = revwalk.set_sorting(git2::Sort::TOPOLOGICAL) {
        return json!(format!("Failed to set sorting: {}", e));
    }
    
    // Configure the revision range
    if let Err(e) = configure_revwalk(
        &repository,
        &mut revwalk,
        props.from_ref.as_deref(),
        props.to_ref.as_deref()
    ) {
        return json!(format!("Failed to configure revision range: {}", e));
    }
    
    // If a path is specified, filter the commits by that path
    let path_spec = props.path.as_deref();
    
    // Collect the commits
    let mut commit_details = Vec::new();
    let mut count = 0;

    for oid in revwalk {
        if count >= limit {
            break;
        }
        
        let oid = match oid {
            Ok(oid) => oid,
            Err(e) => return json!(format!("Error walking revisions: {}", e)),
        };
        
        let commit = match repository.find_commit(oid) {
            Ok(commit) => commit,
            Err(e) => return json!(format!("Error finding commit: {}", e)),
        };
        
        // If a path is specified, check if this commit affects the path
        if let Some(path) = path_spec {
            // Find parent commit (if any)
            let parent = if commit.parent_count() > 0 {
                match commit.parent(0) {
                    Ok(parent) => Some(parent),
                    Err(_) => None,
                }
            } else {
                None
            };
            
            // Get the trees to compare
            let commit_tree = match commit.tree() {
                Ok(tree) => tree,
                Err(e) => return json!(format!("Error getting commit tree: {}", e)),
            };
            
            let parent_tree = match parent {
                Some(parent) => match parent.tree() {
                    Ok(tree) => Some(tree),
                    Err(_) => None,
                },
                None => None,
            };
            
            // Compare the trees to see if the path was modified
            let diff = match parent_tree {
                Some(parent_tree) => repository.diff_tree_to_tree(
                    Some(&parent_tree),
                    Some(&commit_tree),
                    Some(&mut git2::DiffOptions::new().pathspec(path)),
                ),
                None => repository.diff_tree_to_tree(
                    None,
                    Some(&commit_tree),
                    Some(&mut git2::DiffOptions::new().pathspec(path)),
                ),
            };
            
            let diff = match diff {
                Ok(diff) => diff,
                Err(e) => return json!(format!("Error creating diff: {}", e)),
            };
            
            // Skip this commit if it doesn't affect the specified path
            if diff.deltas().len() == 0 {
                continue;
            }
        }
        
        // Extract commit information
        let author = commit.author();
        let timestamp = commit.time().seconds();
        let message = commit.message().unwrap_or("").trim().to_string();
        let short_hash = commit.id().to_string()[..7].to_string();
        let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "Unknown date".to_string());
        
        // Format commit for human-readable version
        let commit_str = format!(
            "({}) by: {} <{}> [{}]\n{}",
            short_hash,
            author.name().unwrap_or("Unknown"), 
            author.email().unwrap_or("no-email"),
            datetime,
            message
        );
        
        commit_details.push(commit_str);
        
        count += 1;
    }
    
    json!(commit_details.join("\n\n"))
}