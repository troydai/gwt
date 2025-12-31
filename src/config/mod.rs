use crate::command;
use anyhow::{Result, anyhow, bail};
use console::Style;
use dialoguer::{Confirm, Input, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_WORKTREE_ROOT: &str = ".gwt_store";
const CONFIG_DIR_NAME: &str = ".gwt";
const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Config {
    Omit,
    Loaded(ConfigData, PathBuf),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ConfigData {
    /// Root directory where all git worktrees will be stored
    pub worktree_root: PathBuf,
}

/// Initialize config - load from file or run interactive setup
pub fn load(cmd: &command::Commands) -> Result<Config> {
    let home = home_dir()?;
    load_with_home(cmd, &home)
}

pub fn setup() -> Result<()> {
    let home = home_dir()?;

    eprintln!("\n{}", Style::new().cyan().bright().apply_to("Initializing gwt configuration..."));
    eprintln!("This will create a configuration file to store your worktree settings.\n");

    let d = prompt_for_config_data(&home)?;

    let config_path = config_file_path(&home);
    d.save(&config_path)?;
    eprintln!("Configuration saved to {}", config_path.display());

    let config = Config::Loaded(d, config_path);
    config.ensure_worktree_root()?;

    Ok(())
}

fn load_with_home(cmd: &command::Commands, home: &Path) -> Result<Config> {
    if let command::Commands::Init { .. } = cmd {
        return Ok(Config::Omit);
    }

    if let command::Commands::Config(command::config::ConfigCommands::Setup) = cmd {
        return Ok(Config::Omit);
    }

    let config_path = config_file_path(home);
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        let result = toml::from_str::<ConfigData>(&content)?;
        return Ok(Config::Loaded(result, config_path));
    }

    eprintln!("gwt configuration not found at {}", config_path.display());

    let should_create = Confirm::with_theme(&prompt_theme())
        .with_prompt("Would you like to create a configuration file now?")
        .default(true)
        .interact()
        .map_err(|e| anyhow!("initialization cancelled: {e}"))?;

    if !should_create {
        bail!("configuration file must be created first.");
    }

    eprintln!("\n{}", Style::new().cyan().bright().apply_to("Setting up gwt configuration..."));
    eprintln!("This will create a configuration file to store your worktree settings.\n");

    let d = prompt_for_config_data(home)?;

    d.save(&config_path)?;
    eprintln!("Configuration saved to {}", config_path.display());

    let config = Config::Loaded(d, config_path);
    config.ensure_worktree_root()?;

    Ok(config)
}

fn prompt_for_config_data(home: &Path) -> Result<ConfigData> {
    eprintln!("Please provide the following configuration:");

    let worktree_root: String = Input::with_theme(&prompt_theme())
        .with_prompt("Worktree root directory")
        .default(default_store_path(home).to_string_lossy().to_string())
        .interact_text()
        .map_err(|e| anyhow!("initialization cancelled: {e}"))?;

    Ok(ConfigData {
        worktree_root: PathBuf::from(worktree_root),
    })
}

impl Config {
    pub fn config_path(&self) -> Option<&str> {
        match self {
            Self::Omit => None,
            Self::Loaded(_, path) => path.to_str(),
        }
    }

    pub fn data(&self) -> Option<&ConfigData> {
        match self {
            Self::Omit => None,
            Self::Loaded(data, _) => Some(data),
        }
    }

    pub fn ensure_worktree_root(&self) -> Result<()> {
        let d = self
            .data()
            .ok_or_else(|| anyhow!("config data not loaded"))?;

        if d.worktree_root.exists() {
            return Ok(());
        }

        let should_create = Confirm::with_theme(&prompt_theme())
            .with_prompt(format!(
                "Worktree root '{}' does not exist. Create it?",
                d.worktree_root.display()
            ))
            .default(true)
            .interact()
            .map_err(|e| anyhow!("initialization cancelled: {e}"))?;

        if !should_create {
            bail!("Worktree root must be created before proceed.")
        }

        fs::create_dir_all(&d.worktree_root)?;
        eprintln!("Created directory: {}", d.worktree_root.display());

        Ok(())
    }
}

impl ConfigData {
    /// Save config to the provided path
    /// Creates the config directory if it doesn't exist
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent().filter(|p| !p.exists()) {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)
            .map_err(|e| anyhow!("failed marshalling config data in toml {e}"))?;
        fs::write(path, contents).map_err(|e| anyhow!("failed to write config file {e}"))?;

        Ok(())
    }
}

/// Returns the path to the config file (~/.gwt/config.toml)
fn config_file_path(home: &Path) -> PathBuf {
    home.join(CONFIG_DIR_NAME).join(CONFIG_FILE_NAME)
}

fn default_store_path(home: &Path) -> PathBuf {
    home.join(DEFAULT_WORKTREE_ROOT)
}

fn home_dir() -> Result<PathBuf> {
    std::env::var("GWT_HOME").map(PathBuf::from).or_else(|_| {
        dirs::home_dir().ok_or_else(|| anyhow!("unexpected error: unable to find home directory"))
    })
}

fn prompt_theme() -> ColorfulTheme {
    ColorfulTheme {
        prompt_style: Style::new().cyan().bright(),
        prompt_prefix: Style::new().cyan().bright().apply_to("?".to_string()),
        success_prefix: Style::new().green().bright().apply_to("✔".to_string()),
        values_style: Style::new().cyan().bright(),
        ..ColorfulTheme::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_data_save_and_load() {
        let dir = tempdir().unwrap();
        let home = dir.path().to_path_buf();

        let data = ConfigData {
            worktree_root: PathBuf::from("/tmp/gwt_test"),
        };

        let config_path = config_file_path(&home);
        data.save(&config_path).unwrap();

        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("worktree_root = \"/tmp/gwt_test\""));

        // Test loading
        let cmd = crate::command::Commands::Sw {
            branch: Some("test".to_string()),
            create: false,
            main: false,
        };
        let loaded = load_with_home(&cmd, &home).unwrap();
        match loaded {
            Config::Loaded(loaded_data, _) => {
                assert_eq!(loaded_data, data);
            }
            _ => panic!("Expected Config::Loaded"),
        }
    }

    #[test]
    fn test_config_omit_for_init() {
        let cmd = crate::command::Commands::Init {
            shell: "bash".to_string(),
        };
        let home = PathBuf::from("/tmp");
        let config = load_with_home(&cmd, &home).unwrap();
        assert_eq!(config, Config::Omit);
        assert!(config.config_path().is_none());
        assert!(config.data().is_none());
    }

    #[test]
    fn test_ensure_worktree_root_exists() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("root");
        fs::create_dir_all(&root).unwrap();

        let config = Config::Loaded(
            ConfigData {
                worktree_root: root,
            },
            PathBuf::from("config.toml"),
        );

        assert!(config.ensure_worktree_root().is_ok());
    }
}
