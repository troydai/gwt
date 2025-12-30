use crate::worktree::WorktreeError;
use std::path::PathBuf;
use std::process::{Command, Output};

pub fn git_cmd() -> String {
    std::env::var("GWT_GIT").unwrap_or_else(|_| "git".to_string())
}

pub fn run_git(args: &[&str]) -> Result<Output, WorktreeError> {
    Command::new(git_cmd())
        .args(args)
        .output()
        .map_err(WorktreeError::Io)
}

/// Return the repository top-level directory (`git rev-parse --show-toplevel`).
pub fn git_toplevel() -> Result<PathBuf, WorktreeError> {
    let output = run_git(&["rev-parse", "--show-toplevel"])?;
    if !output.status.success() {
        return Err(WorktreeError::GitError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(PathBuf::from(stdout.trim()))
}
