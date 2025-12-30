use gwt::generate_init;

pub struct Init {
    pub shell: String,
}

#[derive(Debug, thiserror::Error)]
pub enum InitCommandError {
    #[error("{0}")]
    ShellError(String),
}

pub fn handle_init_command(cmd: &Init) -> Result<(), InitCommandError> {
    match generate_init(&cmd.shell) {
        Ok(s) => {
            println!("{}", s);
            Ok(())
        }
        Err(e) => Err(InitCommandError::ShellError(e)),
    }
}
