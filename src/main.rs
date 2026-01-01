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
        Commands::Ls { full } => command::worktree::list(&config, full),
        Commands::Sw {
            branch,
            create,
            main,
        } => command::worktree::switch(&config, branch.as_deref(), create, main),
        Commands::Rm {
            branch,
            delete_branch,
            force_delete_branch,
        } => command::worktree::remove(&config, &branch, delete_branch, force_delete_branch),
        Commands::Init { shell } => command::shell::handle(&shell),
        Commands::Current => command::current::handle(),
    }
}
