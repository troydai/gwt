use clap::Subcommand;
use std::process::exit;

use crate::config;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// View the current configuration file path and contents
    View,
}

pub fn handle_config_command(cmd: &ConfigCommands) {
    match cmd {
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
    }
}
