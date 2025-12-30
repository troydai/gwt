use crate::config;
use crate::config::Config;
use gwt::{
    WorktreeError, add_worktree, branch_exists, find_worktree_for_branch, git_toplevel,
    list_worktrees,
};
use sha1::{Digest, Sha1};
use std::fs;
use std::path::PathBuf;

pub struct Switch {
    pub branch: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SwitchCommandError {
    #[error("Setup cancelled. Run gwt again to configure.")]
    SetupCancelled,
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),
    #[error("Error ensuring worktree root exists: {0}")]
    WorktreeRootError(String),
    #[error("Branch {0} doesn't exist.")]
    BranchNotFound(String),
    #[error("Could not determine repository name from path {0}")]
    RepoNameError(String),
    #[error("Failed to create worktree: {0}")]
    WorktreeCreateError(String),
    #[error("Git error: {0}")]
    GitError(String),
    #[error("Error listing worktrees: {0}")]
    ListError(WorktreeError),
}

fn compute_worktree_hash(repo_name: &str, branch_name: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(format!("{repo_name}|{branch_name}"));
    let digest = hasher.finalize();
    format!("{digest:x}")[0..8].to_string()
}

fn create_worktree_and_print_path(config: &Config, branch: &str) -> Result<(), SwitchCommandError> {
    let exists = branch_exists(branch).map_err(|e| match e {
        WorktreeError::GitError(s) => SwitchCommandError::GitError(s),
        _ => SwitchCommandError::WorktreeCreateError(e.to_string()),
    })?;
    if !exists {
        return Err(SwitchCommandError::BranchNotFound(branch.to_string()));
    }

    let (target_path, _repo_name) = compute_target_path(config, branch)?;

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            SwitchCommandError::WorktreeRootError(format!(
                "Failed to create directory '{}': {e}",
                parent.display()
            ))
        })?;
    }

    add_worktree(&target_path, branch).map_err(|e| match e {
        WorktreeError::GitError(s) => SwitchCommandError::WorktreeCreateError(s),
        _ => SwitchCommandError::WorktreeCreateError(e.to_string()),
    })?;

    println!("{}", target_path.display());
    Ok(())
}

fn compute_target_path(
    config: &Config,
    branch: &str,
) -> Result<(PathBuf, String), SwitchCommandError> {
    let toplevel = git_toplevel().map_err(|e| match e {
        WorktreeError::GitError(s) => SwitchCommandError::GitError(s),
        _ => SwitchCommandError::WorktreeCreateError(e.to_string()),
    })?;
    let repo_name = toplevel
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| SwitchCommandError::RepoNameError(toplevel.display().to_string()))?
        .to_string();

    let hash = compute_worktree_hash(&repo_name, branch);
    let target_path = config.worktree_root.join(&repo_name).join(hash);
    Ok((target_path, repo_name))
}

pub fn handle_switch_command(cmd: &Switch) -> Result<(), SwitchCommandError> {
    let config = match Config::init() {
        Ok(config) => config,
        Err(config::ConfigError::SetupCancelled) => {
            return Err(SwitchCommandError::SetupCancelled);
        }
        Err(e) => return Err(SwitchCommandError::ConfigError(e)),
    };

    config
        .ensure_worktree_root()
        .map_err(|e| SwitchCommandError::WorktreeRootError(e.to_string()))?;

    match list_worktrees() {
        Ok(wts) => match find_worktree_for_branch(&wts, &cmd.branch) {
            Some(w) => {
                println!("{}", w.path().display());
                Ok(())
            }
            None => create_worktree_and_print_path(&config, &cmd.branch),
        },
        Err(e) => match e {
            WorktreeError::GitError(s) => Err(SwitchCommandError::GitError(s)),
            _ => Err(SwitchCommandError::ListError(e)),
        },
    }
}
