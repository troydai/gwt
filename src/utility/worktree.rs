use console::style;
use std::path::PathBuf;

const MAX_BRANCH_WIDTH: usize = 32;

/// Representation of a Git worktree
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Worktree {
    path: PathBuf,
    head: String,
    branch: Option<String>,
}

#[derive(Clone, Copy)]
pub enum BranchRenderMode {
    Full,
    Truncated(usize),
}

impl Worktree {
    pub fn new(path: PathBuf, head: String, branch: Option<String>) -> Self {
        Self { path, head, branch }
    }

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

    pub fn render(&self, current: &Option<PathBuf>, branch_mode: BranchRenderMode) -> String {
        let is_active = current.as_ref().is_some_and(|cw| cw == self.path());
        let commit = style(&self.head()[..7.min(self.head().len())]).green();
        let branch = self.branch().unwrap_or("(detached)");
        let path = style(self.path().display()).cyan();

        match branch_mode {
            BranchRenderMode::Full => {
                // * b1f0fed fix/issue-76
                //   /Users/troydai/.gwt_store/69fa950d86b47897
                // - 5a37e92 main
                //   /Users/troydai/code/github.com/troydai/gwt
                let marker = if is_active { "*" } else { "-" };
                format!("{} {} {}\n  {}", marker, commit, branch, path)
            }
            BranchRenderMode::Truncated(width) => {
                // truncates the branch name
                let branch_name = if branch.len() <= width {
                    branch.to_string()
                } else {
                    format!("{}..", &branch[..width - 2]) // TODO: I think this is wrong
                };

                // * b1f0fed fix/issue-76 /Users/troydai/.gwt_store/69fa950d86b47897
                //   5a37e92 main         /Users/troydai/code/github.com/troydai/gwt
                let marker = if is_active { "*" } else { " " };
                format!(
                    "{} {} {:<width$} {}",
                    marker,
                    commit,
                    branch_name,
                    path,
                    width = width,
                )
            }
        }
    }
}

/// Representation of a collection of Git worktrees
pub struct Worktrees(Vec<Worktree>);

pub enum ListBranchMode {
    Raw,                // removes the empty ones
    Full(&'static str), // use the default value for the empty ones
}

impl Worktrees {
    pub fn new(trees: Vec<Worktree>) -> Self {
        Self(trees)
    }

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

    pub fn branches(&self, mode: ListBranchMode) -> Vec<String> {
        match mode {
            ListBranchMode::Raw => self
                .iter()
                .filter_map(|wt| wt.branch().map(String::from))
                .collect(),
            ListBranchMode::Full(default) => self
                .iter()
                .map(|wt| {
                    wt.branch()
                        .map_or_else(|| String::from(default), String::from)
                })
                .collect(),
        }
    }

    pub fn max_branch_width(&self) -> usize {
        self.0
            .iter()
            .map(|br| br.branch().map_or_else(|| "(detached)".len(), |s| s.len()))
            .map(|l| l.min(MAX_BRANCH_WIDTH))
            .max()
            .unwrap_or(0)
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

    #[test]
    fn test_worktree_render() {
        console::set_colors_enabled(false);

        let wt = Worktree {
            path: PathBuf::from("/path/to/repo"),
            head: "abc123456789".into(),
            branch: Some("feature-branch".into()),
        };

        // Test Full mode
        let full_output = wt.render(&None, BranchRenderMode::Full);
        assert_eq!(full_output, "- abc1234 feature-branch\n  /path/to/repo");

        // Test Truncated mode
        let trunc_output = wt.render(&None, BranchRenderMode::Truncated(10));
        // feature-branch is 14 chars, width is 10. Truncated to "feature-.." (8 chars + "..")
        assert_eq!(trunc_output, "  abc1234 feature-.. /path/to/repo");

        // Test active worktree
        let active_output = wt.render(
            &Some(PathBuf::from("/path/to/repo")),
            BranchRenderMode::Full,
        );
        assert!(active_output.starts_with("*"));
    }

    #[test]
    fn test_worktrees_branches() {
        let wts = Worktrees(vec![
            Worktree::new(PathBuf::from("/a"), "h1".into(), Some("b1".into())),
            Worktree::new(PathBuf::from("/b"), "h2".into(), None),
        ]);

        let raw_branches = wts.branches(ListBranchMode::Raw);
        assert_eq!(raw_branches, vec!["b1"]);

        let full_branches = wts.branches(ListBranchMode::Full("none"));
        assert_eq!(full_branches, vec!["b1", "none"]);
    }
}
