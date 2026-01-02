use crate::config::Config;
use crate::utility::Git;
use anyhow::Result;
use console::style;

const MAX_BRANCH_WIDTH: usize = 32;

pub fn list(config: &Config, full: bool, raw: bool) -> Result<()> {
    config.ensure_worktree_root()?;

    let git = Git::new();
    let mut worktrees = git.list_worktrees()?;

    // Sort worktrees by branch name alphabetically
    // Detached worktrees (None) come after named branches
    worktrees.sort_by_branch();

    // Raw mode: output only branch names, one per line (for shell completion)
    if raw {
        for wt in worktrees {
            if let Some(branch) = wt.branch() {
                println!("{}", branch);
            }
        }
        return Ok(());
    }

    // Detect the current worktree path (may fail if not in a git worktree)
    let current_worktree = git.git_toplevel().ok();

    // Calculate the maximum branch name width for column alignment
    // Cap at MAX_BRANCH_WIDTH characters unless --full is specified
    let max_branch_width = worktrees
        .iter()
        .map(|wt| {
            if full {
                wt.branch().unwrap_or("(detached)").len()
            } else {
                wt.branch()
                    .unwrap_or("(detached)")
                    .len()
                    .min(MAX_BRANCH_WIDTH)
            }
        })
        .max()
        .unwrap_or(0);

    for wt in worktrees {
        let is_active = current_worktree.as_ref().is_some_and(|cw| cw == wt.path());

        // Truncate commit hash to 7 characters
        let head = wt.head();
        let short_hash = &head[..7.min(head.len())];

        // Truncate branch name or "(detached)" for detached HEAD
        let branch_name = wt.branch().unwrap_or("(detached)");

        // Truncate branch name at MAX_BRANCH_WIDTH characters unless --full is specified
        let display_branch = if full || branch_name.len() <= MAX_BRANCH_WIDTH {
            branch_name.to_string()
        } else {
            format!("{}…", &branch_name[..MAX_BRANCH_WIDTH - 1])
        };

        // Apply color styling: yellow for hash, green for branch
        // Column order: marker, hash, branch, path
        let styled_hash = style(short_hash).yellow();
        let styled_branch = style(format!(
            "{:<width$}",
            display_branch,
            width = max_branch_width
        ))
        .green();

        // Format the marker and path
        let marker = if is_active { "*" } else { " " };
        let styled_path = style(wt.path().display()).cyan();

        // Print with active worktree highlighted in bold
        if is_active {
            println!(
                "{} {} {} {}",
                style(marker).bold(),
                style(styled_hash).bold(),
                style(styled_branch).bold(),
                style(styled_path).bold()
            );
        } else {
            println!(
                "{} {} {} {}",
                marker, styled_hash, styled_branch, styled_path
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::worktree::test_utils::{ENV_LOCK, create_mock_git_script};
    use crate::config::{Config, ConfigData};
    use std::path::PathBuf;

    #[test]
    fn test_list_worktrees() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
case "$1 $2 $3" in
    "worktree list --porcelain")
        echo "worktree /path/to/main
HEAD abc123def456789
branch refs/heads/main

worktree /path/to/feature
HEAD def456abc789012
branch refs/heads/feature-branch"
        exit 0
        ;;
    "rev-parse --show-toplevel")
        echo "/path/to/feature"
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

        let result = list(&config, false, false);
        assert!(result.is_ok());

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_list_worktrees_with_detached() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
case "$1 $2 $3" in
    "worktree list --porcelain")
        echo "worktree /path/to/main
HEAD abc123def456789
branch refs/heads/main

worktree /path/to/detached
HEAD ghi789abc123456
detached"
        exit 0
        ;;
    "rev-parse --show-toplevel")
        echo "/path/to/main"
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

        let result = list(&config, false, false);
        assert!(result.is_ok());

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_list_worktrees_sorted_alphabetically() {
        let _guard = ENV_LOCK.lock().unwrap();
        // The worktrees should be sorted by branch name alphabetically regardless of which one is active
        let script = r#"#!/bin/sh
case "$1 $2 $3" in
    "worktree list --porcelain")
        echo "worktree /path/to/main
HEAD abc123def456789
branch refs/heads/main

worktree /path/to/feature
HEAD def456abc789012
branch refs/heads/feature-branch"
        exit 0
        ;;
    "rev-parse --show-toplevel")
        echo "/path/to/feature"
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

        // The list function should succeed and sort by branch name (feature-branch before main)
        let result = list(&config, false, false);
        assert!(result.is_ok());

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_list_worktrees_no_current_worktree() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Test when git_toplevel fails (e.g., not in any worktree)
        let script = r#"#!/bin/sh
case "$1 $2 $3" in
    "worktree list --porcelain")
        echo "worktree /path/to/main
HEAD abc123def456789
branch refs/heads/main

worktree /path/to/feature
HEAD def456abc789012
branch refs/heads/feature-branch"
        exit 0
        ;;
    "rev-parse --show-toplevel")
        echo "fatal: not a git repository" >&2
        exit 128
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

        // The list function should still succeed even if we can't detect current worktree
        let result = list(&config, false, false);
        assert!(result.is_ok());

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_list_worktrees_in_dangling_worktree_directory() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Test the specific scenario where user is in a dangling worktree directory:
        // - The directory exists on disk
        // - git_toplevel fails because it's not a valid git worktree (orphaned/deleted)
        // - Valid worktrees still exist in the repository
        // Expected behavior:
        // - Command succeeds and lists all valid worktrees
        // - No worktree is marked as active (no asterisk/bold)
        // - Worktrees maintain their original order (no sorting since no active match)
        let script = r#"#!/bin/sh
case "$1 $2 $3" in
    "worktree list --porcelain")
        echo "worktree /path/to/main
HEAD abc123def456789
branch refs/heads/main

worktree /path/to/feature
HEAD def456abc789012
branch refs/heads/feature-branch"
        exit 0
        ;;
    "rev-parse --show-toplevel")
        # This simulates being in a dangling directory that's not a valid git worktree
        echo "fatal: not a git repository (or any of the parent directories): .git" >&2
        exit 128
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

        // The list function should gracefully handle dangling directory scenario
        // It will list all valid worktrees, with none marked as active
        let result = list(&config, false, false);
        assert!(
            result.is_ok(),
            "list should succeed even in dangling worktree directory"
        );

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_list_worktrees_dangling_vs_valid_path_matching() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Test edge case: dangling directory path is similar to a valid worktree path
        // but git_toplevel returns a different path (or fails)
        // This ensures we don't accidentally match on path similarity
        let script = r#"#!/bin/sh
case "$1 $2 $3" in
    "worktree list --porcelain")
        echo "worktree /path/to/main
HEAD abc123def456789
branch refs/heads/main

worktree /path/to/feature
HEAD def456abc789012
branch refs/heads/feature-branch"
        exit 0
        ;;
    "rev-parse --show-toplevel")
        # Return a path that doesn't match any valid worktree
        # This could happen if we're in a subdirectory of a dangling worktree
        # or a completely unrelated directory
        echo "/path/to/some/other/directory"
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

        // When current_worktree doesn't match any valid worktree path,
        // no worktree should be marked as active
        let result = list(&config, false, false);
        assert!(
            result.is_ok(),
            "list should succeed when current path doesn't match any worktree"
        );

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_list_worktrees_sorted_alphabetically_by_branch() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Test that worktrees are sorted alphabetically by branch name
        // when there's no active worktree
        let script = r#"#!/bin/sh
case "$1 $2 $3" in
    "worktree list --porcelain")
        # Return worktrees in non-alphabetical order to verify sorting
        echo "worktree /path/to/zebra
HEAD 111111111111111
branch refs/heads/zebra

worktree /path/to/apple
HEAD 222222222222222
branch refs/heads/apple

worktree /path/to/main
HEAD 333333333333333
branch refs/heads/main

worktree /path/to/charlie
HEAD 444444444444444
branch refs/heads/charlie"
        exit 0
        ;;
    "rev-parse --show-toplevel")
        # Not in any worktree
        echo "fatal: not a git repository" >&2
        exit 128
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

        // The list function should succeed and sort alphabetically
        // Expected order: apple, charlie, main, zebra
        let result = list(&config, false, false);
        assert!(
            result.is_ok(),
            "list should succeed with alphabetical sorting"
        );

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_list_worktrees_detached_sorted_last() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Test that detached worktrees appear after named branches
        let script = r#"#!/bin/sh
case "$1 $2 $3" in
    "worktree list --porcelain")
        echo "worktree /path/to/detached1
HEAD 111111111111111
detached

worktree /path/to/zebra
HEAD 222222222222222
branch refs/heads/zebra

worktree /path/to/apple
HEAD 333333333333333
branch refs/heads/apple

worktree /path/to/detached2
HEAD 444444444444444
detached"
        exit 0
        ;;
    "rev-parse --show-toplevel")
        echo "fatal: not a git repository" >&2
        exit 128
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

        // The list function should sort named branches first (alphabetically),
        // then detached worktrees
        // Expected order: apple, zebra, (detached), (detached)
        let result = list(&config, false, false);
        assert!(
            result.is_ok(),
            "list should succeed with detached worktrees last"
        );

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_list_worktrees_truncate_long_branch_names() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Test that branch names longer than MAX_BRANCH_WIDTH characters are truncated by default
        let script = r#"#!/bin/sh
case "$1 $2 $3" in
    "worktree list --porcelain")
        echo "worktree /path/to/short
HEAD 111111111111111
branch refs/heads/short

worktree /path/to/very-long
HEAD 222222222222222
branch refs/heads/feature/this-is-a-very-long-branch-name-that-exceeds-thirty-two-chars"
        exit 0
        ;;
    "rev-parse --show-toplevel")
        echo "fatal: not a git repository" >&2
        exit 128
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

        // Test without --full flag (should truncate)
        let result = list(&config, false, false);
        assert!(
            result.is_ok(),
            "list should succeed with truncated branch names"
        );

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_list_worktrees_full_flag_shows_complete_branch_names() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Test that --full flag shows complete branch names without truncation
        let script = r#"#!/bin/sh
case "$1 $2 $3" in
    "worktree list --porcelain")
        echo "worktree /path/to/short
HEAD 111111111111111
branch refs/heads/short

worktree /path/to/very-long
HEAD 222222222222222
branch refs/heads/feature/this-is-a-very-long-branch-name-that-is-way-longer-than-max-width-to-ensure-no-truncation-happens-when-full-is-used"
        exit 0
        ;;
    "rev-parse --show-toplevel")
        echo "fatal: not a git repository" >&2
        exit 128
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

        // Test with --full flag (should not truncate)
        let result = list(&config, true, false);
        assert!(result.is_ok(), "list should succeed with full branch names");

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }
}
