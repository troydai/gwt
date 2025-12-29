use dialoguer::{Confirm, Input};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config file not found at {0}")]
    NotFound(PathBuf),
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Failed to serialize config: {0}")]
    SerializeError(#[from] toml::ser::Error),
    #[error("Interactive setup error: {0}")]
    InteractiveError(String),
    #[error("Setup cancelled by user")]
    SetupCancelled,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Root directory where all git worktrees will be stored
    pub worktree_root: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        // Default to ~/.gwt_store
        let home = dirs::home_dir().expect("Could not determine home directory");
        Self {
            worktree_root: home.join(".gwt_store"),
        }
    }
}

/// Returns the path to the gwt config directory (~/.gwt)
pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".gwt")
}

/// Returns the path to the config file (~/.gwt/config.toml)
pub fn config_file_path() -> PathBuf {
    config_dir().join("config.toml")
}

impl Config {
    /// Load config from the default config file path
    pub fn load() -> Result<Self, ConfigError> {
        let path = config_file_path();
        if !path.exists() {
            return Err(ConfigError::NotFound(path));
        }
        let contents = fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save config to the default config file path
    /// Creates the config directory if it doesn't exist
    pub fn save(&self) -> Result<(), ConfigError> {
        let dir = config_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        let path = config_file_path();
        let contents = toml::to_string_pretty(self).map_err(ConfigError::SerializeError)?;
        fs::write(&path, contents)?;
        Ok(())
    }

    /// Interactively prompt user to create initial configuration
    pub fn interactive_setup() -> Result<Self, ConfigError> {
        let config_path = config_file_path();

        println!("gwt configuration not found at {}", config_path.display());

        let should_create = Confirm::new()
            .with_prompt("Would you like to create a configuration file now?")
            .default(true)
            .interact()
            .map_err(|e| ConfigError::InteractiveError(e.to_string()))?;

        if !should_create {
            return Err(ConfigError::SetupCancelled);
        }

        let default_root = dirs::home_dir()
            .map(|h| h.join(".gwt_store"))
            .unwrap_or_else(|| PathBuf::from("~/.gwt_store"));

        let worktree_root: String = Input::new()
            .with_prompt("Worktree root directory")
            .default(default_root.to_string_lossy().to_string())
            .interact_text()
            .map_err(|e| ConfigError::InteractiveError(e.to_string()))?;

        let config = Config {
            worktree_root: PathBuf::from(worktree_root),
        };

        config.save()?;
        println!("Configuration saved to {}", config_path.display());

        Ok(config)
    }

    /// Initialize config - load from file or run interactive setup
    pub fn init() -> Result<Self, ConfigError> {
        match Self::load() {
            Ok(config) => Ok(config),
            Err(ConfigError::NotFound(_)) => Self::interactive_setup(),
            Err(e) => Err(e),
        }
    }

    /// Ensure the worktree root directory exists
    pub fn ensure_worktree_root(&self) -> Result<(), ConfigError> {
        if !self.worktree_root.exists() {
            let should_create = Confirm::new()
                .with_prompt(format!(
                    "Worktree root '{}' does not exist. Create it?",
                    self.worktree_root.display()
                ))
                .default(true)
                .interact()
                .map_err(|e| ConfigError::InteractiveError(e.to_string()))?;

            if should_create {
                fs::create_dir_all(&self.worktree_root)?;
                println!("Created directory: {}", self.worktree_root.display());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = Config {
            worktree_root: PathBuf::from("/home/user/worktrees"),
        };
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.worktree_root, parsed.worktree_root);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(
            config
                .worktree_root
                .to_string_lossy()
                .contains(".gwt_store")
        );
    }

    #[test]
    fn test_config_toml_format() {
        let config = Config {
            worktree_root: PathBuf::from("/test/path"),
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("worktree_root"));
        assert!(toml_str.contains("/test/path"));
    }
}
