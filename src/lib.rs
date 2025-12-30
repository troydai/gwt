mod shell;
mod utility;
mod worktree;

// Re-export a minimal set of symbols for the binary to use.
pub use shell::generate_init;
pub use utility::git_toplevel;
pub use worktree::{
    Worktree, WorktreeError, add_worktree, branch_exists, find_worktree_for_branch, list_worktrees,
};
