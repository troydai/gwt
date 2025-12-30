use crate::config::Config;
use anyhow::{Context, Result, anyhow, bail};
use sha1::{Digest, Sha1};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub fn handle(config: &Config, branch: &str) -> Result<()> {
    config.ensure_worktree_root()?;

    let wts = list_worktrees()?;

    match find_worktree_for_branch(&wts, branch) {
        Some(w) => {
            println!("{}", w.path().display());
            Ok(())
        }
        None => create_worktree_and_print_path(config, branch),
    }
}

fn compute_worktree_hash(repo_name: &str, branch_name: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(format!("{repo_name}|{branch_name}"));
    let digest = hasher.finalize();
    format!("{digest:x}")[0..8].to_string()
}

fn create_worktree_and_print_path(config: &Config, branch: &str) -> Result<()> {
    let exists = branch_exists(branch).context("Failed to check if branch exists")?;
    if !exists {
        bail!("Branch '{}' doesn't exist.", branch);
    }

    let target_path = compute_target_path(config, branch)?;

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory '{}'", parent.display()))?;
    }

    add_worktree(&target_path, branch).context("Failed to add worktree")?;

    println!("{}", target_path.display());
    Ok(())
}

fn compute_target_path(config: &Config, branch: &str) -> Result<PathBuf> {
    let toplevel = git_toplevel().context("Failed to get git toplevel")?;
    let repo_name = toplevel
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            anyhow!(
                "Could not determine repository name from path {}",
                toplevel.display()
            )
        })?
        .to_string();

    let hash = compute_worktree_hash(&repo_name, branch);
    let worktree_root = config
        .data()
        .map(|d| &d.worktree_root)
        .ok_or_else(|| anyhow!("Config not loaded"))?;
    let target_path = worktree_root.join(&repo_name).join(hash);
    Ok(target_path)
}

/// Representation of a Git worktree
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Worktree {
    path: PathBuf,
    head: String,
    branch: Option<String>,
}

impl Worktree {
    /// Return the worktree path
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Return the head SHA
    #[cfg(test)]
    pub fn head(&self) -> &str {
        &self.head
    }

    /// Return branch name, if any
    #[cfg(test)]
    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }
}

fn parse_porcelain(input: &str) -> Vec<Worktree> {
    let mut worktrees = Vec::new();

    let mut current_path: Option<PathBuf> = None;
    let mut current_head: Option<String> = None;
    let mut current_branch: Option<String> = None;

    for line in input.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            // finalize current block
            if let (Some(path), Some(head)) = (current_path.take(), current_head.take()) {
                worktrees.push(Worktree {
                    path,
                    head,
                    branch: current_branch.take(),
                });
            }
            current_path = None;
            current_head = None;
            current_branch = None;
            continue;
        }

        if let Some(rest) = line.strip_prefix("worktree ") {
            current_path = Some(PathBuf::from(rest));
        } else if let Some(rest) = line.strip_prefix("HEAD ") {
            current_head = Some(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("branch ") {
            // branch may be in the form refs/heads/<name>
            let branch_name = rest.strip_prefix("refs/heads/").unwrap_or(rest).to_string();
            current_branch = Some(branch_name);
        } else if line == "detached" {
            current_branch = None;
        }
    }

    // finalize last block if any
    if let (Some(path), Some(head)) = (current_path.take(), current_head.take()) {
        worktrees.push(Worktree {
            path,
            head,
            branch: current_branch.take(),
        });
    }

    worktrees
}

pub fn list_worktrees() -> Result<Vec<Worktree>> {
    let git_cmd = std::env::var("GWT_GIT").unwrap_or_else(|_| "git".to_string());
    list_worktrees_with_cmd(&git_cmd)
}

fn list_worktrees_with_cmd(git_cmd: &str) -> Result<Vec<Worktree>> {
    let output = Command::new(git_cmd)
        .args(["worktree", "list", "--porcelain"])
        .output()
        .map_err(|e| anyhow!("failed git execution: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        bail!("failed git execution: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_porcelain(&stdout))
}

pub fn find_worktree_for_branch<'a>(
    worktrees: &'a [Worktree],
    branch: &str,
) -> Option<&'a Worktree> {
    worktrees.iter().find(|w| match &w.branch {
        Some(b) => b == branch,
        None => false,
    })
}

// Helper functions

fn git_cmd() -> String {
    std::env::var("GWT_GIT").unwrap_or_else(|_| "git".to_string())
}

fn run_git(args: &[&str]) -> Result<Output> {
    Command::new(git_cmd())
        .args(args)
        .output()
        .map_err(|e| anyhow!("IO error: {e}"))
}

fn branch_exists(branch: &str) -> Result<bool> {
    let ref_name = format!("refs/heads/{branch}");
    let output = run_git(&["show-ref", "--verify", "--quiet", &ref_name])?;
    if output.status.success() {
        return Ok(true);
    }
    if output.status.code().is_some_and(|c| c == 1) {
        return Ok(false);
    }
    bail!("Git error: {}", String::from_utf8_lossy(&output.stderr))
}

fn add_worktree(path: &Path, branch: &str) -> Result<()> {
    let output = Command::new(git_cmd())
        .args(["worktree", "add"])
        .arg(path)
        .arg(branch)
        .output()
        .map_err(|e| anyhow!("IO error: {e}"))?;

    if !output.status.success() {
        bail!("Git error: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

fn git_toplevel() -> Result<PathBuf> {
    let output = run_git(&["rev-parse", "--show-toplevel"])?;
    if !output.status.success() {
        bail!("Git error: {}", String::from_utf8_lossy(&output.stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(PathBuf::from(stdout.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigData;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn parse_porcelain_two_worktrees() {
        let input = "worktree /path/to/main
HEAD abc123
branch refs/heads/main

worktree /path/to/feature
HEAD def456
branch refs/heads/feature-branch
";

        let parsed = parse_porcelain(input);
        assert_eq!(parsed.len(), 2);

        assert_eq!(parsed[0].path(), &PathBuf::from("/path/to/main"));
        assert_eq!(parsed[0].head(), "abc123");
        assert_eq!(parsed[0].branch(), Some("main"));

        assert_eq!(parsed[1].path(), &PathBuf::from("/path/to/feature"));
        assert_eq!(parsed[1].head(), "def456");
        assert_eq!(parsed[1].branch(), Some("feature-branch"));
    }

    #[test]
    fn parse_porcelain_detached_worktree() {
        let input = "worktree /path/to/detached
HEAD ghi789
detached
";

        let parsed = parse_porcelain(input);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].path(), &PathBuf::from("/path/to/detached"));
        assert_eq!(parsed[0].head(), "ghi789");
        assert_eq!(parsed[0].branch(), None);
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
        assert_eq!(w.path(), &PathBuf::from("/path/to/feature"));
    }

    #[test]
    fn parse_branch_with_slash() {
        let input = "worktree /path/to/feature
HEAD abcabc
branch refs/heads/feature/my-feature
";

        let parsed = parse_porcelain(input);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].branch(), Some("feature/my-feature"));
    }

    #[test]
    fn parse_multiple_blocks_last_block_without_trailing_blank_correct() {
        let input = "worktree /a
HEAD a1
branch refs/heads/a

worktree /b
HEAD b1
branch refs/heads/b";
        let parsed = parse_porcelain(input);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[1].branch(), Some("b"));
    }

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
    fn test_list_worktrees_with_mock_git() {
        let script = r#"#!/bin/sh
echo "worktree /path/to/main
HEAD abc123
branch refs/heads/main"
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        let wts = list_worktrees_with_cmd(mock_git.to_str().unwrap()).unwrap();
        assert_eq!(wts.len(), 1);
        assert_eq!(wts[0].branch(), Some("main"));
    }

    #[test]
    fn test_compute_worktree_hash() {
        let hash = compute_worktree_hash("my-repo", "my-feature");
        assert_eq!(hash.len(), 8);
        assert_eq!(compute_worktree_hash("my-repo", "my-feature"), hash);
        assert_ne!(compute_worktree_hash("my-repo", "other-feature"), hash);
    }

    #[test]
    fn test_branch_exists_true() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
if [ "$1" = "show-ref" ] && [ "$2" = "--verify" ] && [ "$3" = "--quiet" ] && [ "$4" = "refs/heads/existing-branch" ]; then
    exit 0
else
    echo "unexpected args: $@" >&2
    exit 1
fi
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", mock_git);
        }

        assert!(branch_exists("existing-branch").unwrap());
        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_branch_exists_false() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
if [ "$1" = "show-ref" ]; then
    exit 1
fi
exit 1
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", mock_git);
        }

        assert!(!branch_exists("non-existent").unwrap());
        unsafe {
            std::env::remove_var("GWT_GIT");
        }
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
            std::env::set_var("GWT_GIT", mock_git);
        }

        let config = Config::Loaded(
            ConfigData {
                worktree_root: PathBuf::from("/tmp/wt-root"),
            },
            PathBuf::from("/tmp/config"),
        );

        let path = compute_target_path(&config, "feature-branch").unwrap();

        let hash = compute_worktree_hash("my-repo", "feature-branch");
        let expected_path = PathBuf::from("/tmp/wt-root/my-repo").join(hash);
        assert_eq!(path, expected_path);

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }
}
