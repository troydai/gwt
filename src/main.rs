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

fn handle_error(e: impl std::fmt::Display) -> ! {
    eprintln!("{}", e);
    exit(1);
}

fn main() {
    let cli = Cli::parse();

    // Initialize config for all commands except Init (will prompt if missing)
    // Note: clap handles --help and help subcommand before we reach here
    if !matches!(&cli.command, Commands::Init { .. }) {
        config::Config::init()
            .map_err(|e| {
                if matches!(e, config::ConfigError::SetupCancelled) {
                    handle_error("Setup cancelled. Run gwt again to configure.");
                } else {
                    handle_error(format!("Configuration error: {}", e));
                }
            })
            .ok();
    }

    match &cli.command {
        Commands::Config(config_cmd) => {
            command::config::handle_config_command(config_cmd)
                .map_err(handle_error)
                .ok();
        }
        Commands::Switch { branch } => {
            command::worktree::handle_switch_command(&command::worktree::Switch {
                branch: branch.clone(),
            })
            .map_err(handle_error)
            .ok();
        }
        Commands::Init { shell } => {
            command::shell::handle_init_command(&command::shell::Init {
                shell: shell.clone(),
            })
            .map_err(handle_error)
            .ok();
        }
    }
}
