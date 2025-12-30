pub mod config;
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

    /// Switch to an existing worktree for a branch (prints path on success)
    Sw {
        /// Branch name to switch to
        branch: String,
    },

    /// Output shell integration code for a given shell (bash, zsh, fish)
    Init {
        /// Shell name
        shell: String,
    },
}
