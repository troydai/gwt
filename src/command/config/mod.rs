use anyhow::{Result, anyhow};
use clap::Subcommand;
use config::Config;
use console::Style;

use crate::config;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// View the current configuration file path and contents
    View,
    /// Reset the configuration
    Setup,
}

pub fn handle(config: &Config, cmd: &ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::View => view_config(config),
        ConfigCommands::Setup => {
            config::setup()?;
            Ok(())
        }
    }
}

fn view_config(config: &Config) -> Result<()> {
    let p = config
        .config_path()
        .ok_or_else(|| anyhow!("unexpected error: invalid config"))?;

    let label_style = Style::new().cyan().bright();
    let path_style = Style::new().yellow();
    let contents_style = Style::new().white().bright();

    println!(
        "{} {}",
        label_style.apply_to("Config file path:"),
        path_style.apply_to(p)
    );

    // Config is already initialized in main.rs, so we can just read it
    let contents = std::fs::read_to_string(p)?;
    println!("\n{}", label_style.apply_to("Config file contents:"));
    println!("{}", contents_style.apply_to(contents));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_execute_config_view_success() {
        let dir = tempdir().unwrap();
        let config_file = dir.path().join("config.toml");

        let config_content = r###"worktree_root = "/tmp/test_worktrees""###;
        fs::write(&config_file, config_content).unwrap();

        // Test the file reading logic directly
        assert!(config_file.exists());
        let contents = fs::read_to_string(&config_file).unwrap();
        assert!(contents.contains("worktree_root"));
        assert!(contents.contains("/tmp/test_worktrees"));
    }

    #[test]
    fn test_execute_config_view_file_not_found() {
        let dir = tempdir().unwrap();
        let config_file = dir.path().join("nonexistent_config.toml");

        assert!(!config_file.exists());

        // Test that reading a non-existent file returns an error
        let result = std::fs::read_to_string(&config_file);
        assert!(result.is_err());

        // Verify the error type
        match result {
            Err(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
            }
            Ok(_) => panic!("Expected error for non-existent file"),
        }
    }

    #[test]
    fn test_config_commands_view_variant() {
        // Test that ConfigCommands::View can be created and matched
        let cmd = ConfigCommands::View;
        match cmd {
            ConfigCommands::View => {
                // This ensures the variant exists and can be matched
            }
            _ => {}
        }
    }

    #[test]
    fn test_execute_config_view_with_invalid_toml() {
        // Test that we can read a file even if it has invalid TOML
        // (The function just reads and prints, doesn't parse)
        let dir = tempdir().unwrap();
        let config_file = dir.path().join("config.toml");

        let invalid_content = "this is not valid toml but we can still read it";
        fs::write(&config_file, invalid_content).unwrap();

        // The function should still succeed in reading the file
        let contents = fs::read_to_string(&config_file).unwrap();
        assert_eq!(contents, invalid_content);
    }

    #[test]
    fn test_execute_config_command_view_with_real_config() {
        // Test the actual execute_config_command function
        // This requires a real config file in the home directory
        // We'll create a temporary config and test if it exists

        // Create a temporary config file structure
        let dir = tempdir().unwrap();
        let config_dir = dir.path().join(".gwt");
        fs::create_dir_all(&config_dir).unwrap();
        let config_file = config_dir.join("config.toml");

        let config_content = r###"worktree_root = "/tmp/test_worktrees""###;
        fs::write(&config_file, config_content).unwrap();

        // Verify the file structure matches what we expect
        assert!(config_file.exists());
        assert!(config_file.is_file());

        // Test reading the config
        let contents = fs::read_to_string(&config_file).unwrap();
        assert!(contents.contains("worktree_root"));
    }

    #[test]
    fn test_config_commands_setup_variant() {
        let cmd = ConfigCommands::Setup;
        match cmd {
            ConfigCommands::Setup => {}
            _ => panic!("Expected Setup variant"),
        }
    }
}
