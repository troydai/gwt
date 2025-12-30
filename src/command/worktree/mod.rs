use crate::config;
use crate::config::Config;
use gwt::{WorktreeError, find_worktree_for_branch, list_worktrees};

pub struct Switch {
    pub branch: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SwitchCommandError {
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),
    #[error("Worktree for branch {0} doesn't exist.")]
    WorktreeNotFound(String),
    #[error("Git error: {0}")]
    GitError(String),
    #[error("Error listing worktrees: {0}")]
    ListError(WorktreeError),
}

pub fn handle_switch_command(config: &Config, cmd: &Switch) -> Result<(), SwitchCommandError> {
    config.ensure_worktree_root()?;

    match list_worktrees() {
        Ok(wts) => match find_worktree_for_branch(&wts, &cmd.branch) {
            Some(w) => {
                println!("{}", w.path().display());
                Ok(())
            }
            None => Err(SwitchCommandError::WorktreeNotFound(cmd.branch.clone())),
        },
        Err(e) => match e {
            WorktreeError::GitError(s) => Err(SwitchCommandError::GitError(s)),
            _ => Err(SwitchCommandError::ListError(e)),
        },
    }
}
