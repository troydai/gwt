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

fn handle_result<T, E: std::fmt::Display>(result: Result<T, E>) {
    if let Err(e) = result {
        handle_error(e);
    }
}

fn main() {
    let cli = Cli::parse();

    // Initialize config for all commands except Init (will prompt if missing)
    // Note: clap handles --help and help subcommand before we reach here
    if !matches!(&cli.command, Commands::Init { .. }) {
        match config::Config::init() {
            Err(config::ConfigError::SetupCancelled) => {
                handle_error("Setup cancelled. Run gwt again to configure.");
            }
            Err(e) => {
                handle_error(format!("Configuration error: {}", e));
            }
            Ok(_) => {}
        }
    }

    match &cli.command {
        Commands::Config(config_cmd) => {
            handle_result(command::config::handle_config_command(config_cmd));
        }
        Commands::Switch { branch } => {
            let switch_cmd = command::worktree::Switch {
                branch: branch.clone(),
            };
            handle_result(command::worktree::handle_switch_command(&switch_cmd));
        }
        Commands::Init { shell } => {
            let init_cmd = command::shell::Init {
                shell: shell.clone(),
            };
            handle_result(command::shell::handle_init_command(&init_cmd));
        }
    }
}
