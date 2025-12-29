use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

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
    pub fn head(&self) -> &str {
        &self.head
    }

    /// Return branch name, if any
    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }
}

/// Errors from worktree operations
#[derive(Debug, Error)]
pub(crate) enum WorktreeError {
    #[error("Git command failed: {0}")]
    GitError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse git output: {0}")]
    ParseError(String),
}

/// Parse `git worktree list --porcelain` output into a Vec<Worktree>
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

/// Run `git worktree list --porcelain` and return parsed worktrees
///
/// The `GWT_GIT` environment variable can be used to override the `git` executable
/// (useful for tests to inject a mock git program).
pub(crate) fn list_worktrees() -> Result<Vec<Worktree>, WorktreeError> {
    let git_cmd = std::env::var("GWT_GIT").unwrap_or_else(|_| "git".to_string());

    let output = Command::new(&git_cmd)
        .args(["worktree", "list", "--porcelain"])
        .output()
        .map_err(WorktreeError::Io)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(WorktreeError::GitError(stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_porcelain(&stdout))
}

/// Find a worktree by branch name from a slice of worktrees
pub(crate) fn find_worktree_for_branch<'a>(worktrees: &'a [Worktree], branch: &str) -> Option<&'a Worktree> {
    worktrees.iter().find(|w| match &w.branch {
        Some(b) => b == branch,
        None => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
}
