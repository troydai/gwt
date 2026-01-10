pub mod completion;
pub mod config;
pub mod current;
pub mod home;
pub mod shell;
pub mod worktree;

use clap::{Parser, Subcommand, ValueEnum};

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
    Ls {
        /// Show full branch names without truncation
        #[arg(long = "full")]
        full: bool,

        /// Output only branch names, one per line (for shell completion)
        #[arg(long = "raw", hide = true)]
        raw: bool,
    },

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

        /// Specify the remote to resolve ambiguity when multiple remotes have the same branch name
        #[arg(long = "remote")]
        remote: Option<String>,
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

    /// Generate shell completion scripts
    Completion {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: ShellType,
    },

    /// Switch to the home worktree (original repository)
    Home,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
}
