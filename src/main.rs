mod command;
mod config;

use clap::{Parser, Subcommand};
use std::process::exit;

#[derive(Parser)]
#[command(name = "gwt")]
#[command(about = "A git worktree manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure gwt
    #[command(subcommand)]
    Config(command::config::ConfigCommands),

    /// Switch to an existing worktree for a branch (prints path on success)
    Switch {
        /// Branch name to switch to
        branch: String,
    },

    /// Output shell integration code for a given shell (bash, zsh, fish)
    Init {
        /// Shell name
        shell: String,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        if let Some(config::ConfigError::SetupCancelled) = e.downcast_ref::<config::ConfigError>() {
            eprintln!("Setup cancelled. Run gwt again to configure.");
        } else {
            eprintln!("{}", e);
        }
        exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    if let Commands::Init { shell } = cli.command {
        return command::shell::handle_init_command(&command::shell::Init { shell })
            .map_err(|e| e.into());
    }

    let config = config::Config::init()?;

    match cli.command {
        Commands::Config(config_cmd) => {
            command::config::handle_config_command(&config, &config_cmd)?;
        }
        Commands::Switch { branch } => {
            command::worktree::handle_switch_command(
                &config,
                &command::worktree::Switch { branch },
            )?;
        }
        Commands::Init { .. } => unreachable!(),
    }
    Ok(())
}
