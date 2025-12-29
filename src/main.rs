mod config;

use clap::{Parser, Subcommand};
use config::Config;
use std::process::exit;

use gwt::{WorktreeError, find_worktree_for_branch, generate_init, list_worktrees};

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
    Config(ConfigCommands),

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

#[derive(Subcommand)]
enum ConfigCommands {
    /// View the current configuration file path and contents
    View,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Config(config_cmd) => match config_cmd {
            ConfigCommands::View => {
                let config_path = match config::config_file_path() {
                    Ok(path) => path,
                    Err(e) => {
                        eprintln!("Error getting config file path: {}", e);
                        exit(1);
                    }
                };

                println!("Config file path: {}", config_path.display());

                if !config_path.exists() {
                    eprintln!("Config file does not exist at {}", config_path.display());
                    exit(1);
                }

                match std::fs::read_to_string(&config_path) {
                    Ok(contents) => {
                        println!("\nConfig file contents:");
                        println!("{}", contents);
                    }
                    Err(e) => {
                        eprintln!("Error reading config file: {}", e);
                        exit(1);
                    }
                }
            }
        },
        Commands::Switch { branch } => {
            let config = match Config::init() {
                Ok(config) => config,
                Err(config::ConfigError::SetupCancelled) => {
                    eprintln!("Setup cancelled. Run gwt again to configure.");
                    exit(1);
                }
                Err(e) => {
                    eprintln!("Configuration error: {}", e);
                    exit(1);
                }
            };

            if let Err(e) = config.ensure_worktree_root() {
                eprintln!("Error ensuring worktree root exists: {}", e);
                exit(1);
            }

            match list_worktrees() {
                Ok(wts) => match find_worktree_for_branch(&wts, branch) {
                    Some(w) => {
                        println!("{}", w.path().display());
                        exit(0);
                    }
                    None => {
                        eprintln!("Worktree for branch {} doesn't exist.", branch);
                        exit(1);
                    }
                },
                Err(e) => match e {
                    WorktreeError::GitError(s) => {
                        eprintln!("Git error: {}", s);
                        exit(1);
                    }
                    _ => {
                        eprintln!("Error listing worktrees: {}", e);
                        exit(1);
                    }
                },
            }
        },
        Commands::Init { shell } => match generate_init(shell) {
            Ok(s) => {
                println!("{}", s);
            }
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        },
    }
}
