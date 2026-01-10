use crate::utility::Git;
use anyhow::Result;

pub fn handle() -> Result<()> {
    let git = Git::new();
    let home = git.get_main_worktree()?;
    println!("{}", home.path().display());
    Ok(())
}
