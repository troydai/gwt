use crate::utility::Git;
use anyhow::Result;

pub fn handle() -> Result<()> {
    let git = Git::new();

    let branch = git.get_current_branch()?;
    let toplevel = git.git_toplevel()?;

    let branch_name = if branch.is_empty() {
        "(detached)".to_string()
    } else {
        branch
    };

    println!(
        "Branch {} @ Worktree {}",
        branch_name,
        toplevel.display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_handle_with_regular_branch() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
if [ "$1" = "branch" ] && [ "$2" = "--show-current" ]; then
    echo "main"
    exit 0
elif [ "$1" = "rev-parse" ] && [ "$2" = "--show-toplevel" ]; then
    echo "/path/to/repo"
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

        let result = handle();
        assert!(result.is_ok());

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_handle_with_detached_head() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
if [ "$1" = "branch" ] && [ "$2" = "--show-current" ]; then
    echo ""
    exit 0
elif [ "$1" = "rev-parse" ] && [ "$2" = "--show-toplevel" ]; then
    echo "/path/to/repo"
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

        let result = handle();
        assert!(result.is_ok());

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }

    #[test]
    fn test_handle_with_feature_branch() {
        let _guard = ENV_LOCK.lock().unwrap();
        let script = r#"#!/bin/sh
if [ "$1" = "branch" ] && [ "$2" = "--show-current" ]; then
    echo "feature/my-feature"
    exit 0
elif [ "$1" = "rev-parse" ] && [ "$2" = "--show-toplevel" ]; then
    echo "/home/user/projects/my-repo"
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

        let result = handle();
        assert!(result.is_ok());

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }
}
