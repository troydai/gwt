use console::Style;
use dialoguer::{Confirm, Input, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

const DEFAULT_WORKTREE_ROOT: &str = ".gwt_store";
const CONFIG_DIR_NAME: &str = ".gwt";
const CONFIG_FILE_NAME: &str = "config.toml";

/// Returns a configured ColorfulTheme with bright colors for interactive prompts
fn prompt_theme() -> ColorfulTheme {
    ColorfulTheme {
        prompt_style: Style::new().cyan().bright(),
        prompt_prefix: Style::new().cyan().bright().apply_to("?".to_string()),
        success_prefix: Style::new().green().bright().apply_to("✔".to_string()),
        values_style: Style::new().cyan().bright(),
        ..ColorfulTheme::default()
    }
}

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
    #[error("Could not determine home directory")]
    HomeDirNotFound,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Root directory where all git worktrees will be stored
    pub worktree_root: PathBuf,
}

impl Config {
    pub fn new_default() -> Result<Self, ConfigError> {
        let home = dirs::home_dir().ok_or(ConfigError::HomeDirNotFound)?;
        Ok(Self {
            worktree_root: home.join(DEFAULT_WORKTREE_ROOT),
        })
    }
}

/// Returns the path to the gwt config directory (~/.gwt)
pub fn config_dir() -> Result<PathBuf, ConfigError> {
    dirs::home_dir()
        .ok_or(ConfigError::HomeDirNotFound)
        .map(|p| p.join(CONFIG_DIR_NAME))
}

/// Returns the path to the config file (~/.gwt/config.toml)
pub fn config_file_path() -> Result<PathBuf, ConfigError> {
    Ok(config_dir()?.join(CONFIG_FILE_NAME))
}

impl Config {
    /// Load config from a specific path
    pub fn load_from(path: &PathBuf) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::NotFound(path.clone()));
        }
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Load config from the default config file path
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from(&config_file_path()?)
    }

    /// Save config to a specific path
    pub fn save_to(&self, path: &PathBuf) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent().filter(|p| !p.exists()) {
            fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self).map_err(ConfigError::SerializeError)?;
        fs::write(path, contents)?;
        Ok(())
    }

    /// Save config to the default config file path
    /// Creates the config directory if it doesn't exist
    pub fn save(&self) -> Result<(), ConfigError> {
        Self::save_to(self, &config_file_path()?)
    }

    /// Interactively prompt user to create initial configuration
    pub fn interactive_setup() -> Result<Self, ConfigError> {
        let config_path = config_file_path()?;

        println!("gwt configuration not found at {}", config_path.display());

        let should_create = Confirm::with_theme(&prompt_theme())
            .with_prompt("Would you like to create a configuration file now?")
            .default(true)
            .interact()
            .map_err(|e| ConfigError::InteractiveError(e.to_string()))?;

        if !should_create {
            return Err(ConfigError::SetupCancelled);
        }

        let default_root = Self::new_default()
            .map(|c| c.worktree_root)
            .unwrap_or_else(|_| PathBuf::from(format!("~/{}", DEFAULT_WORKTREE_ROOT)));

        let worktree_root: String = Input::with_theme(&prompt_theme())
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
            let should_create = Confirm::with_theme(&prompt_theme())
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
    fn test_new_default() {
        if let Ok(config) = Config::new_default() {
            assert!(
                config
                    .worktree_root
                    .to_string_lossy()
                    .contains(DEFAULT_WORKTREE_ROOT)
            );
        }
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

    #[test]
    fn test_save_and_load_with_tempfile() {
        let dir = tempfile::tempdir().unwrap();
        let config_file = dir.path().join("config.toml");

        let config = Config {
            worktree_root: dir.path().join("worktrees"),
        };

        config.save_to(&config_file).unwrap();
        assert!(config_file.exists());

        let loaded_config = Config::load_from(&config_file).unwrap();
        assert_eq!(config, loaded_config);
    }
}
