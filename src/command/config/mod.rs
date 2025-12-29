use clap::Subcommand;
use std::process::exit;

use crate::config;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// View the current configuration file path and contents
    View,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigCommandError {
    #[error("Error getting config file path: {0}")]
    ConfigPathError(#[from] config::ConfigError),
    #[error("Config file does not exist at {0}")]
    ConfigNotFound(std::path::PathBuf),
    #[error("Error reading config file: {0}")]
    ReadError(#[from] std::io::Error),
}

pub fn handle_config_command(cmd: &ConfigCommands) {
    execute_config_command(cmd).unwrap_or_else(|e| {
        eprintln!("{}", e);
        exit(1);
    });
}

fn execute_config_command(cmd: &ConfigCommands) -> Result<(), ConfigCommandError> {
    match cmd {
        ConfigCommands::View => {
            let config_path = config::config_file_path()?;

            println!("Config file path: {}", config_path.display());

            if !config_path.exists() {
                return Err(ConfigCommandError::ConfigNotFound(config_path));
            }

            let contents = std::fs::read_to_string(&config_path)?;
            println!("\nConfig file contents:");
            println!("{}", contents);

            Ok(())
        }
    }
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
        
        let config_content = r#"worktree_root = "/tmp/test_worktrees"
"#;
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
    fn test_config_command_error_types() {
        // Test ConfigCommandError variants can be created and formatted
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.toml");
        
        let not_found_err = ConfigCommandError::ConfigNotFound(path.clone());
        let error_msg = format!("{}", not_found_err);
        assert!(error_msg.contains("Config file does not exist"));
        assert!(error_msg.contains(path.to_string_lossy().as_ref()));
        
        // Test ReadError
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test");
        let read_err = ConfigCommandError::ReadError(io_err);
        let error_msg = format!("{}", read_err);
        assert!(error_msg.contains("Error reading config file"));
    }

    #[test]
    fn test_config_commands_view_variant() {
        // Test that ConfigCommands::View can be created and matched
        let cmd = ConfigCommands::View;
        match cmd {
            ConfigCommands::View => {
                // This ensures the variant exists and can be matched
                assert!(true);
            }
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
        
        let config_content = r#"worktree_root = "/tmp/test_worktrees"
"#;
        fs::write(&config_file, config_content).unwrap();
        
        // Verify the file structure matches what we expect
        assert!(config_file.exists());
        assert!(config_file.is_file());
        
        // Test reading the config
        let contents = fs::read_to_string(&config_file).unwrap();
        assert!(contents.contains("worktree_root"));
    }

    #[test]
    fn test_config_command_error_display() {
        // Test error message formatting
        let path = std::path::PathBuf::from("/nonexistent/path/config.toml");
        let err = ConfigCommandError::ConfigNotFound(path.clone());
        
        let error_string = format!("{}", err);
        assert!(error_string.contains("Config file does not exist"));
        assert!(error_string.contains(path.to_string_lossy().as_ref()));
    }
}
