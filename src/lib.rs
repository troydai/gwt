mod shell;
mod worktree;

// Re-export a minimal set of symbols for the binary to use.
pub use shell::generate_init;
pub use worktree::{Worktree, WorktreeError, find_worktree_for_branch, list_worktrees};
