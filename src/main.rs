mod command;
mod config;
mod utility;

use crate::command::{Cli, Commands};
use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::load(&cli.command)?;

    match cli.command {
        Commands::Config(config_command) => command::config::handle(&config, &config_command),
        Commands::Ls { full, raw } => command::worktree::list(&config, full, raw),
        Commands::Sw {
            branch,
            create,
            main,
            remote,
        } => command::worktree::switch(&config, branch.as_deref(), create, main, remote.as_deref()),
        Commands::Rm {
            branch,
            delete_branch,
            force_delete_branch,
        } => command::worktree::remove(&config, &branch, delete_branch, force_delete_branch),
        Commands::Init { shell } => command::shell::handle(&shell),
        Commands::Current => command::current::handle(),
        Commands::Completion { shell } => command::completion::handle(shell),
    }
}
