mod config;

use clap::{Parser, Subcommand};
use config::Config;

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
    Config,
}

fn main() {
    let config = match Config::init() {
        Ok(config) => config,
        Err(config::ConfigError::SetupCancelled) => {
            eprintln!("Setup cancelled. Run gwt again to configure.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = config.ensure_worktree_root() {
        eprintln!("Error ensuring worktree root exists: {}", e);
        std::process::exit(1);
    }

    let cli = Cli::parse();

    match &cli.command {
        Commands::Config => {
            println!("Current configuration:");
            println!("Worktree root: {}", config.worktree_root.display());
        }
    }
}
