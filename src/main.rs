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

    // Initialize config for all commands except Init (will prompt if missing)
    // Note: clap handles --help and help subcommand before we reach here
    if !matches!(&cli.command, Commands::Init { .. }) {
        if let Err(e) = config::Config::init() {
            let msg = match e {
                config::ConfigError::SetupCancelled => "Setup cancelled. Run gwt again to configure.",
                _ => return eprintln!("Configuration error: {}", e),
            };
            eprintln!("{}", msg);
            exit(1);
        }
    }

    let result = match &cli.command {
        Commands::Config(config_cmd) => command::config::handle_config_command(config_cmd),
        Commands::Switch { branch } => {
            command::worktree::handle_switch_command(&command::worktree::Switch {
                branch: branch.clone(),
            })
        }
        Commands::Init { shell } => {
            command::shell::handle_init_command(&command::shell::Init {
                shell: shell.clone(),
            })
        }
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        exit(1);
    }
}
