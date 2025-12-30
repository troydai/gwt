pub mod config;
pub mod current;
pub mod shell;
pub mod worktree;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gwt")]
#[command(about = "A git worktree manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Configure gwt
    #[command(subcommand)]
    Config(config::ConfigCommands),

    /// List all worktrees
    Ls,

    /// Switch to an existing worktree for a branch (prints path on success)
    Sw {
        /// Branch name to switch to
        branch: String,

        /// Create a new branch
        #[arg(short = 'b', long = "create-branch")]
        create: bool,
    },

    /// Remove a worktree by branch name
    Rm {
        /// Branch name of the worktree to remove
        branch: String,

        /// Delete the branch after removing the worktree
        #[arg(short = 'b', long = "delete-branch")]
        delete_branch: bool,

        /// Force delete the branch (use -D instead of -d)
        #[arg(short = 'B', long = "force-delete-branch")]
        force_delete_branch: bool,
    },

    /// Output shell integration code for a given shell (bash, zsh, fish)
    Init {
        /// Shell name
        shell: String,
    },

    /// Print current worktree and branch information
    #[command(alias = "c")]
    Current,
}
