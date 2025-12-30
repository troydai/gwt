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
        Commands::Sw { branch, create } => command::worktree::handle(&config, &branch, create),
        Commands::Init { shell } => command::shell::handle(&shell),
    }
}
