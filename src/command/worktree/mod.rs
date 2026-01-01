mod list;

pub use list::list;

use crate::config::Config;
use crate::utility::Git;
use anyhow::{Context, Result, anyhow, bail};
use sha1::{Digest, Sha1};
use std::env;
use std::fs;
use std::path::PathBuf;

use console::{Term, style};
use dialoguer::Confirm;

pub fn switch(config: &Config, branch: Option<&str>, create: bool, use_main: bool) -> Result<()> {
    config.ensure_worktree_root()?;

    let git = Git::new();

    // Resolve the branch name based on the flag
    let target_branch = if use_main {
        resolve_main_branch(&git)?
    } else {
        branch
            .ok_or_else(|| anyhow!("Branch name is required"))?
            .to_string()
    };

    if git.get_current_branch().is_ok_and(|c| c == target_branch) {
        eprintln!(
            "{}",
            style(format!("You are already on branch '{}'.", target_branch)).yellow()
        );
        std::process::exit(1);
    }

    let wt_path = git
        .list_worktrees()?
        .iter()
        .find(|wt| wt.branch().is_some_and(|v| v == target_branch))
        .map(|wt| wt.path().clone())
        .map_or_else(
            || create_worktree_and_print_path(&git, config, &target_branch, create),
            Ok,
        )?;

    println!("{}", wt_path.display());
    Ok(())
}

fn create_worktree_and_print_path(
    git: &Git,
    config: &Config,
    branch: &str,
    create: bool,
) -> Result<PathBuf> {
    let exists = git
        .branch_exists(branch)
        .context("Failed to check if branch exists")?;
    if !exists {
        if create {
            git.create_branch(branch)
                .context(format!("Failed to create branch '{}'", branch))?;
            eprintln!("Branch '{}' created.", branch);
        } else {
            bail!("Branch '{}' doesn't exist.", branch);
        }
    }

    let target_path = compute_target_path(git, config, branch)?;

    // If the target path already exists but is not a valid worktree, fail with instructions
    // (we know it's not a valid worktree because we didn't find it in list_worktrees)
    if target_path.exists() {
        bail!(
            "Cannot create worktree: directory '{}' already exists.\n\n\
            This is likely an orphaned worktree directory from a previous operation.\n\
            To resolve this, remove the directory manually:\n\n\
            \trm -rf '{}'\n\n\
            Then try again.",
            target_path.display(),
            target_path.display()
        );
    }

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory '{}'", parent.display()))?;
    }

    let target_path_str = target_path
        .to_str()
        .ok_or_else(|| anyhow!("Invalid target path"))?;

    git.add_worktree(target_path_str, branch)
        .context("Failed to add worktree")?;

    eprintln!(
        "Created worktree for branch '{}' at '{}'",
        branch,
        target_path.display()
    );

    Ok(target_path)
}

pub fn remove(
    config: &Config,
    branch: &str,
    delete_branch: bool,
    force_delete_branch: bool,
) -> Result<()> {
    config.ensure_worktree_root()?;

    let git = Git::new();

    // Find the worktree for this branch
    let worktree = git
        .find_worktree_by_branch(branch)?
        .ok_or_else(|| anyhow!("No worktree found for branch '{}'", branch))?;

    let worktree_path = worktree.path();

    // Check if we're currently in the worktree being removed
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let need_to_switch = current_dir.starts_with(worktree_path);

    // Get the main worktree path if we need to switch (but don't print yet)
    let main_path = if need_to_switch {
        Some(git.get_main_worktree()?.path().clone())
    } else {
        None
    };

    // Request confirmation
    let prompt = format!(
        "Remove worktree at '{}' for branch '{}'?",
        worktree_path.display(),
        branch
    );

    let confirmed = Confirm::new()
        .with_prompt(prompt)
        .default(false)
        .interact_on(&Term::stderr())
        .context("Failed to get confirmation")?;

    if !confirmed {
        eprintln!("Removal cancelled.");
        return Ok(());
    }

    // Print the main worktree path so the shell wrapper can cd to it (only after confirmation)
    if let Some(path) = main_path {
        println!("{}", path.display());
    }

    // Remove the worktree
    let worktree_path_str = worktree_path
        .to_str()
        .ok_or_else(|| anyhow!("Invalid worktree path"))?;

    git.remove_worktree(worktree_path_str)
        .context("Failed to remove worktree")?;

    eprintln!("Worktree for branch '{}' removed.", branch);

    // Delete the branch if requested
    if delete_branch || force_delete_branch {
        git.delete_branch(branch, force_delete_branch)
            .context("Failed to delete branch")?;
        eprintln!("Branch '{}' deleted.", branch);
    }

    Ok(())
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

fn resolve_main_branch(git: &Git) -> Result<String> {
    // Check if 'main' exists first
    if git.branch_exists("main")? {
        return Ok("main".to_string());
    }

    // Fall back to 'master'
    if git.branch_exists("master")? {
        return Ok("master".to_string());
    }

    bail!("Neither 'main' nor 'master' branch exists")
}

// Helper functions

#[cfg(test)]
pub(crate) mod test_utils {
    use std::path::PathBuf;
    use std::sync::Mutex;
    use tempfile::tempdir;

    pub(crate) static ENV_LOCK: Mutex<()> = Mutex::new(());

    pub(crate) fn create_mock_git_script(script_content: &str) -> (PathBuf, tempfile::TempDir) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::worktree::test_utils::{ENV_LOCK, create_mock_git_script};
    use crate::config::ConfigData;
    use std::path::PathBuf;

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
        let wt_root = _dir.path().join("wt-root");
        std::fs::create_dir_all(&wt_root).unwrap();

        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let config = Config::Loaded(
            ConfigData {
                worktree_root: wt_root.clone(),
            },
            PathBuf::from("/tmp/config"),
        );

        let git = Git::new();
        let path = compute_target_path(&git, &config, "feature-branch").unwrap();

        let hash = compute_worktree_hash("/path/to/my-repo", "feature-branch");
        let expected_path = wt_root.join(hash);
        assert_eq!(path, expected_path);

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_create_worktree_and_print_path_with_create() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
case "$@" in
    "for-each-ref --format=%(refname) refs/heads/new-branch")
        exit 0
        ;;
    "branch new-branch")
        exit 0
        ;;
    "rev-parse --show-toplevel")
        echo "/path/to/repo"
        exit 0
        ;;
    "worktree add "* )
        exit 0
        ;;
    *)
        echo "unexpected args: $@" >&2
        exit 1
        ;;
esac
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        let wt_root = _dir.path().join("wt-root");
        std::fs::create_dir_all(&wt_root).unwrap();

        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let config = Config::Loaded(
            ConfigData {
                worktree_root: wt_root,
            },
            PathBuf::from("/tmp/config"),
        );

        let git = Git::new();
        let result = create_worktree_and_print_path(&git, &config, "new-branch", true);
        assert!(result.is_ok());

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_resolve_main_branch_when_only_main_exists() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
case "$@" in
    "for-each-ref --format=%(refname) refs/heads/main")
        echo "refs/heads/main"
        exit 0
        ;;
    "for-each-ref --format=%(refname) refs/heads/master")
        exit 0
        ;;
    *)
        echo "unexpected args: $@" >&2
        exit 1
        ;;
esac
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let git = Git::new();
        let result = resolve_main_branch(&git);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "main");

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_resolve_main_branch_when_only_master_exists() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
case "$@" in
    "for-each-ref --format=%(refname) refs/heads/main")
        exit 0
        ;;
    "for-each-ref --format=%(refname) refs/heads/master")
        echo "refs/heads/master"
        exit 0
        ;;
    *)
        echo "unexpected args: $@" >&2
        exit 1
        ;;
esac
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let git = Git::new();
        let result = resolve_main_branch(&git);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "master");

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_resolve_main_branch_when_both_exist_prefer_main() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
case "$@" in
    "for-each-ref --format=%(refname) refs/heads/main")
        echo "refs/heads/main"
        exit 0
        ;;
    "for-each-ref --format=%(refname) refs/heads/master")
        echo "refs/heads/master"
        exit 0
        ;;
    *)
        echo "unexpected args: $@" >&2
        exit 1
        ;;
esac
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let git = Git::new();
        let result = resolve_main_branch(&git);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "main");

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_resolve_main_branch_when_neither_exists() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
case "$@" in
    "for-each-ref --format=%(refname) refs/heads/main")
        exit 0
        ;;
    "for-each-ref --format=%(refname) refs/heads/master")
        exit 0
        ;;
    *)
        echo "unexpected args: $@" >&2
        exit 1
        ;;
esac
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let git = Git::new();
        let result = resolve_main_branch(&git);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Neither 'main' nor 'master' branch exists")
        );

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }
}
