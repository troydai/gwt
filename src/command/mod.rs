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
        #[arg(required_unless_present = "main", conflicts_with = "main")]
        branch: Option<String>,

        /// Create a new branch
        #[arg(short = 'b', long = "create-branch")]
        create: bool,

        /// Switch to the main branch (main or master)
        #[arg(short = 'm', long = "main")]
        main: bool,
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
