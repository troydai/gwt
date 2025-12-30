use crate::config::Config;
use crate::utility::Git;
use crate::utility::Worktree;
use anyhow::{Context, Result, anyhow, bail};
use sha1::{Digest, Sha1};
use std::fs;
use std::path::PathBuf;

pub fn handle(config: &Config, branch: &str) -> Result<()> {
    config.ensure_worktree_root()?;

    let git = Git::new();
    let wts = git.list_worktrees()?;

    let wt_path = match find_worktree_for_branch(&wts, branch) {
        Some(path) => path,
        None => create_worktree_and_print_path(&git, config, branch)?,
    };

    println!("{}", wt_path.display());
    Ok(())
}

fn create_worktree_and_print_path(git: &Git, config: &Config, branch: &str) -> Result<PathBuf> {
    let exists = git
        .branch_exists(branch)
        .context("Failed to check if branch exists")?;
    if !exists {
        bail!("Branch '{}' doesn't exist.", branch);
    }

    let target_path = compute_target_path(git, config, branch)?;
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory '{}'", parent.display()))?;
    }

    let target_path_str = target_path
        .to_str()
        .ok_or_else(|| anyhow!("Invalid target path"))?;

    git.add_worktree(target_path_str, branch)
        .context("Failed to add worktree")?;

    Ok(target_path)
}

fn compute_target_path(git: &Git, config: &Config, branch: &str) -> Result<PathBuf> {
    let toplevel = git.git_toplevel().context("Failed to get git toplevel")?;
    let repo_path = toplevel
        .to_str()
        .ok_or_else(|| anyhow!("invalid toplevel path"))?;

    let hash = compute_worktree_hash(repo_path, branch);
    let worktree_root = config
        .data()
        .map(|d| &d.worktree_root)
        .ok_or_else(|| anyhow!("Config not loaded"))?;
    let target_path = worktree_root.join(hash);
    Ok(target_path)
}

fn compute_worktree_hash(repo_name: &str, branch_name: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(format!("{repo_name}|{branch_name}"));
    let digest = hasher.finalize();
    format!("{digest:x}")[0..16].to_string()
}

pub fn find_worktree_for_branch(worktrees: &[Worktree], branch: &str) -> Option<PathBuf> {
    worktrees
        .iter()
        .find(|wt| wt.branch().is_some_and(|v| v == branch))
        .map(|wt| wt.path().clone())
}

// Helper functions

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigData;
    use crate::utility::parse_porcelain;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn create_mock_git_script(script_content: &str) -> (PathBuf, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let mock_git = dir.path().join("mock-git");
        std::fs::write(&mock_git, script_content).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&mock_git, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        (mock_git, dir)
    }

    #[test]
    fn find_worktree_by_branch() {
        let input = "worktree /path/to/main
HEAD abc123
branch refs/heads/main

worktree /path/to/feature
HEAD def456
branch refs/heads/feature-branch
";

        let parsed = parse_porcelain(input);
        let found = find_worktree_for_branch(&parsed, "feature-branch");
        assert!(found.is_some());
        let w = found.unwrap();
        assert_eq!(w, PathBuf::from("/path/to/feature"));
    }

    #[test]
    fn test_compute_worktree_hash() {
        let hash = compute_worktree_hash("my-repo", "my-feature");
        assert_eq!(hash.len(), 16);
        assert_eq!(compute_worktree_hash("my-repo", "my-feature"), hash);
        assert_ne!(compute_worktree_hash("my-repo", "other-feature"), hash);
    }

    #[test]
    fn test_compute_target_path() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Mock git toplevel
        let script = r#"#!/bin/sh
if [ "$1" = "rev-parse" ] && [ "$2" = "--show-toplevel" ]; then
    echo "/path/to/my-repo"
    exit 0
fi
exit 1
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let config = Config::Loaded(
            ConfigData {
                worktree_root: PathBuf::from("/tmp/wt-root"),
            },
            PathBuf::from("/tmp/config"),
        );

        let git = Git::new();
        let path = compute_target_path(&git, &config, "feature-branch").unwrap();

        let hash = compute_worktree_hash("/path/to/my-repo", "feature-branch");
        let expected_path = PathBuf::from("/tmp/wt-root").join(hash);
        assert_eq!(path, expected_path);

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }
}
