mod shell;
mod worktree;

// Re-export a minimal set of symbols for the binary to use.
pub use worktree::{Worktree, WorktreeError, list_worktrees, find_worktree_for_branch};
pub use shell::generate_init;
