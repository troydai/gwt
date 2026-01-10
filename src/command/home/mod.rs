use crate::utility::Git;
use anyhow::Result;

pub fn handle() -> Result<()> {
    let git = Git::new();
    let home = git.get_main_worktree()?;
    println!("{}", home.path().display());
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
    fn test_handle_returns_main_worktree_path() {
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

        let result = handle();
        assert!(result.is_ok());

        unsafe {
            std::env::remove_var("GWT_GIT");
        }
    }
}
