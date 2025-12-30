use crate::config;
use crate::config::Config;
use gwt::generate_init;

pub struct Init {
    pub shell: String,
}

#[derive(Debug, thiserror::Error)]
pub enum InitCommandError {
    #[error("Setup cancelled. Run gwt again to configure.")]
    SetupCancelled,
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),
    #[error("{0}")]
    ShellError(String),
}

pub fn handle_init_command(cmd: &Init) -> Result<(), InitCommandError> {
    // Ensure config exists (will prompt if missing)
    match Config::init() {
        Ok(_) => {}
        Err(config::ConfigError::SetupCancelled) => {
            return Err(InitCommandError::SetupCancelled);
        }
        Err(e) => return Err(InitCommandError::ConfigError(e)),
    }

    match generate_init(&cmd.shell) {
        Ok(s) => {
            println!("{}", s);
            Ok(())
        }
        Err(e) => Err(InitCommandError::ShellError(e)),
    }
}
