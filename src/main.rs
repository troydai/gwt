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

    match &cli.command {
        Commands::Config(config_cmd) => {
            if let Err(e) = command::config::handle_config_command(config_cmd) {
                eprintln!("{}", e);
                exit(1);
            }
        }
        Commands::Switch { branch } => {
            let switch_cmd = command::worktree::Switch {
                branch: branch.clone(),
            };
            if let Err(e) = command::worktree::handle_switch_command(&switch_cmd) {
                eprintln!("{}", e);
                exit(1);
            }
        }
        Commands::Init { shell } => {
            let init_cmd = command::shell::Init {
                shell: shell.clone(),
            };
            if let Err(e) = command::shell::handle_init_command(&init_cmd) {
                eprintln!("{}", e);
                exit(1);
            }
        }
    }
}
