use anyhow::{Result, anyhow, bail};
use std::{
    path::PathBuf,
    process::{Command, Output},
};

pub struct Git {
    exec: String,
}

impl Git {
    pub fn new() -> Self {
        Self {
            exec: std::env::var("GWT_GIT").unwrap_or_else(|_| "git".to_string()),
        }
    }

    pub fn run(&self, args: &[&str]) -> Result<Output> {
        let output = Command::new(&self.exec)
            .args(args)
            .output()
            .map_err(|e| anyhow!("git error: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("git error: {stderr}");
        }
        Ok(output)
    }

    pub fn get_current_branch(&self) -> Result<String> {
        let output = self.run(&["branch", "--show-current"])?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }

    pub fn list_worktrees(&self) -> Result<Worktrees> {
        let output = self.run(&["worktree", "list", "--porcelain"])?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_porcelain(&stdout))
    }

    pub fn branch_exists(&self, branch: &str) -> Result<bool> {
        let ref_name = format!("refs/heads/{branch}");
        let output = self.run(&["for-each-ref", "--format=%(refname)", &ref_name])?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().any(|line| line.trim() == ref_name))
    }

    pub fn remote_branch_exists(&self, remote_branch: &str) -> Result<bool> {
        let ref_name = format!("refs/remotes/{remote_branch}");
        // Git stores remote branches as references in 'refs/remotes/'.
        // We use 'for-each-ref' to search for the specific full reference path.
        // If the branch exists, 'for-each-ref' will print the reference name.
        // If it doesn't exist, it will produce no output. This is more reliable
        // than 'git branch -r' which is designed for human-readable output.
        let output = self.run(&["for-each-ref", "--format=%(refname)", &ref_name])?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().any(|line| line.trim() == ref_name))
    }

    pub fn find_remote_branches_by_name(&self, name: &str) -> Result<Vec<String>> {
        let pattern = format!("refs/remotes/*/{}", name);
        let output = self.run(&["for-each-ref", "--format=%(refname)", &pattern])?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .map(|line| {
                line.trim()
                    .strip_prefix("refs/remotes/")
                    .unwrap_or(line)
                    .to_string()
            })
            .collect())
    }

    pub fn create_branch(&self, branch: &str) -> Result<()> {
        self.run(&["branch", branch])?;
        Ok(())
    }

    pub fn create_branch_from_remote(&self, local: &str, remote: &str) -> Result<()> {
        self.run(&["branch", "--track", local, remote])?;
        Ok(())
    }

    pub fn add_worktree(&self, path: &str, branch: &str) -> Result<()> {
        self.run(&["worktree", "add", path, branch])?;
        Ok(())
    }

    pub fn git_toplevel(&self) -> Result<PathBuf> {
        let output = self.run(&["rev-parse", "--show-toplevel"])?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(PathBuf::from(stdout.trim()))
    }

    pub fn remove_worktree(&self, path: &str) -> Result<()> {
        self.run(&["worktree", "remove", path])?;
        Ok(())
    }

    pub fn delete_branch(&self, branch: &str, force: bool) -> Result<()> {
        let flag = if force { "-D" } else { "-d" };
        self.run(&["branch", flag, branch])?;
        Ok(())
    }

    pub fn get_main_worktree(&self) -> Result<Worktree> {
        let worktrees = self.list_worktrees()?;
        worktrees
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("No worktrees found"))
    }

    pub fn find_worktree_by_branch(&self, branch: &str) -> Result<Option<Worktree>> {
        let worktrees = self.list_worktrees()?;
        Ok(worktrees
            .into_iter()
            .find(|wt| wt.branch().is_some_and(|b| b == branch)))
    }
}

pub(crate) fn parse_porcelain(input: &str) -> Worktrees {
    let mut trees = Vec::new();

    let mut current_path: Option<PathBuf> = None;
    let mut current_head: Option<String> = None;
    let mut current_branch: Option<String> = None;

    for line in input.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            // finalize current block
            if let (Some(path), Some(head)) = (current_path.take(), current_head.take()) {
                trees.push(Worktree {
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
        trees.push(Worktree {
            path,
            head,
            branch: current_branch.take(),
        });
    }

    Worktrees(trees)
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
    #[allow(dead_code)]
    pub fn head(&self) -> &str {
        &self.head
    }

    /// Return branch name, if any
    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }
}

pub struct Worktrees(Vec<Worktree>);

impl Worktrees {
    /// Sort worktrees by branch name alphabetically.
    /// Detached worktrees (None) come after named branches.
    pub fn sort_by_branch(&mut self) {
        self.0.sort_by(|a, b| match (a.branch(), b.branch()) {
            (Some(a_branch), Some(b_branch)) => a_branch.cmp(b_branch),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
    }
}

impl std::ops::Deref for Worktrees {
    type Target = Vec<Worktree>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Worktrees {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Worktrees {
    type Item = Worktree;
    type IntoIter = std::vec::IntoIter<Worktree>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Worktrees {
    type Item = &'a Worktree;
    type IntoIter = std::slice::Iter<'a, Worktree>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn test_list_worktrees_with_mock_git() {
        let script = r#"#!/bin/sh
if [ "$1" = "worktree" ] && [ "$2" = "list" ] && [ "$3" = "--porcelain" ]; then
    echo "worktree /path/to/main
HEAD abc123
branch refs/heads/main"
else
    echo "unexpected args: $@" >&2
    exit 1
fi
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        // We need to inject the mock git path.
        // Since Git::new() reads from env, we can set env var.
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let git = Git::new();
        let wts = git.list_worktrees().unwrap();

        unsafe {
            std::env::remove_var("GWT_GIT");
        }

        assert_eq!(wts.len(), 1);
        assert_eq!(wts[0].branch(), Some("main"));
    }

    #[test]
    fn test_branch_exists_true() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
if [ "$1" = "for-each-ref" ] && [ "$2" = "--format=%(refname)" ] && [ "$3" = "refs/heads/existing-branch" ]; then
    echo "refs/heads/existing-branch"
    exit 0
else
    echo "unexpected args: $@" >&2
    exit 1
fi
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let git = Git::new();
        assert!(git.branch_exists("existing-branch").unwrap());

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_branch_exists_false() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
if [ "$1" = "for-each-ref" ] && [ "$2" = "--format=%(refname)" ] && [ "$3" = "refs/heads/non-existent" ]; then
    exit 0
else
    echo "unexpected args: $@" >&2
    exit 1
fi
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let git = Git::new();
        assert!(!git.branch_exists("non-existent").unwrap());

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_create_branch() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
if [ "$1" = "branch" ] && [ "$2" = "new-branch" ]; then
    exit 0
else
    echo "unexpected args: $@" >&2
    exit 1
fi
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let git = Git::new();
        assert!(git.create_branch("new-branch").is_ok());

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_remove_worktree() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
if [ "$1" = "worktree" ] && [ "$2" = "remove" ] && [ "$3" = "/path/to/worktree" ]; then
    exit 0
else
    echo "unexpected args: $@" >&2
    exit 1
fi
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let git = Git::new();
        assert!(git.remove_worktree("/path/to/worktree").is_ok());

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_delete_branch() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
if [ "$1" = "branch" ] && [ "$2" = "-d" ] && [ "$3" = "my-branch" ]; then
    exit 0
else
    echo "unexpected args: $@" >&2
    exit 1
fi
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let git = Git::new();
        assert!(git.delete_branch("my-branch", false).is_ok());

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_delete_branch_force() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
if [ "$1" = "branch" ] && [ "$2" = "-D" ] && [ "$3" = "my-branch" ]; then
    exit 0
else
    echo "unexpected args: $@" >&2
    exit 1
fi
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let git = Git::new();
        assert!(git.delete_branch("my-branch", true).is_ok());

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_get_main_worktree() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
if [ "$1" = "worktree" ] && [ "$2" = "list" ] && [ "$3" = "--porcelain" ]; then
    echo "worktree /path/to/main
HEAD abc123
branch refs/heads/main

worktree /path/to/feature
HEAD def456
branch refs/heads/feature"
    exit 0
else
    echo "unexpected args: $@" >&2
    exit 1
fi
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let git = Git::new();
        let main_wt = git.get_main_worktree().unwrap();
        assert_eq!(main_wt.path(), &PathBuf::from("/path/to/main"));
        assert_eq!(main_wt.branch(), Some("main"));

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_find_worktree_by_branch() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
if [ "$1" = "worktree" ] && [ "$2" = "list" ] && [ "$3" = "--porcelain" ]; then
    echo "worktree /path/to/main
HEAD abc123
branch refs/heads/main

worktree /path/to/feature
HEAD def456
branch refs/heads/feature-branch"
    exit 0
else
    echo "unexpected args: $@" >&2
    exit 1
fi
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let git = Git::new();
        let wt = git.find_worktree_by_branch("feature-branch").unwrap();
        assert!(wt.is_some());
        assert_eq!(wt.unwrap().path(), &PathBuf::from("/path/to/feature"));

        let wt2 = git.find_worktree_by_branch("non-existent").unwrap();
        assert!(wt2.is_none());

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_find_remote_branches_by_name() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
if [ "$1" = "for-each-ref" ] && [ "$2" = "--format=%(refname)" ] && [ "$3" = "refs/remotes/*/my-branch" ]; then
    echo "refs/remotes/origin/my-branch"
    echo "refs/remotes/upstream/my-branch"
    exit 0
else
    echo "unexpected args: $@" >&2
    exit 1
fi
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let git = Git::new();
        let matches = git.find_remote_branches_by_name("my-branch").unwrap();
        assert_eq!(matches.len(), 2);
        assert!(matches.contains(&"origin/my-branch".to_string()));
        assert!(matches.contains(&"upstream/my-branch".to_string()));

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_remote_branch_exists_true() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
if [ "$1" = "for-each-ref" ] && [ "$2" = "--format=%(refname)" ] && [ "$3" = "refs/remotes/origin/existing-remote" ]; then
    echo "refs/remotes/origin/existing-remote"
    exit 0
else
    echo "unexpected args: $@" >&2
    exit 1
fi
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let git = Git::new();
        assert!(git.remote_branch_exists("origin/existing-remote").unwrap());

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_remote_branch_exists_false() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
if [ "$1" = "for-each-ref" ] && [ "$2" = "--format=%(refname)" ] && [ "$3" = "refs/remotes/origin/non-existent" ]; then
    exit 0
else
    echo "unexpected args: $@" >&2
    exit 1
fi
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let git = Git::new();
        assert!(!git.remote_branch_exists("origin/non-existent").unwrap());

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_create_branch_from_remote() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
if [ "$1" = "branch" ] && [ "$2" = "--track" ] && [ "$3" = "local-branch" ] && [ "$4" = "origin/remote-branch" ]; then
    exit 0
else
    echo "unexpected args: $@" >&2
    exit 1
fi
"#;
        let (mock_git, _dir) = create_mock_git_script(script);
        unsafe {
            std::env::set_var("GWT_GIT", &mock_git);
        }

        let git = Git::new();
        assert!(
            git.create_branch_from_remote("local-branch", "origin/remote-branch")
                .is_ok()
        );

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_worktrees_sorting_and_deref() {
        let mut wts = Worktrees(vec![
            Worktree {
                path: PathBuf::from("/z"),
                head: "h1".into(),
                branch: Some("zebra".into()),
            },
            Worktree {
                path: PathBuf::from("/d"),
                head: "h2".into(),
                branch: None,
            },
            Worktree {
                path: PathBuf::from("/a"),
                head: "h3".into(),
                branch: Some("apple".into()),
            },
        ]);

        // Test Deref (accessing .len() from Vec)
        assert_eq!(wts.len(), 3);

        wts.sort_by_branch();

        // Check order: apple, zebra, None
        assert_eq!(wts[0].branch(), Some("apple"));
        assert_eq!(wts[1].branch(), Some("zebra"));
        assert_eq!(wts[2].branch(), None);
    }

    #[test]
    fn test_worktrees_into_iterator() {
        let wts = Worktrees(vec![Worktree {
            path: PathBuf::from("/a"),
            head: "h1".into(),
            branch: Some("b1".into()),
        }]);

        // Test IntoIterator for &Worktrees
        let mut count = 0;
        for wt in &wts {
            assert_eq!(wt.branch(), Some("b1"));
            count += 1;
        }
        assert_eq!(count, 1);

        // Test IntoIterator for Worktrees (owned)
        for wt in wts {
            assert_eq!(wt.branch(), Some("b1"));
        }
    }
}
