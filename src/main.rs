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
        Commands::Config(config_cmd) => command::config::handle(&config, &config_cmd),
        Commands::Switch { branch } => command::worktree::handle(&config, &branch),
        Commands::Init { shell } => command::shell::handle(&shell),
    }
}
