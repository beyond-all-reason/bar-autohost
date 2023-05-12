use std::io;
use std::path::Path;
use std::process::Command;
use std::result::Result;

use thiserror::Error;

use crate::utils::config::Config;

#[derive(Error, Debug)]
pub enum LaunchError {
    #[error("Spring failed to launch")]
    LaunchFail(#[from] io::Error),
}

const SPRING_WRITEDIR_ENV_VAR: &str = "SPRING_WRITEDIR";

pub trait Spring {
    fn launch(&self, config: &dyn Config, root_dir: &Path) -> Result<(), LaunchError>;
}

/// A Helper struct for launching `spring-headless` processes.
#[derive(Default)]
pub struct SpringHeadless {}

impl SpringHeadless {
    pub fn new() -> Self {
        SpringHeadless {}
    }
}

impl Spring for SpringHeadless {
    /// Launch `spring-headless` as a detached process.
    ///
    /// The paths in the autohost config file are expected to be relative to the autohost
    /// root directory. The directory the autohost executable lives in.
    /// Absolute paths to spring, `SPRING_WRITEDIR` and start script are created from the
    /// root_dir and the config relative paths.
    ///
    /// # Errors
    ///
    /// A `LaunchError` is returned if `spring-headless` cannot be launched as a detached
    /// process. This can happen for various reasons, such as a permissions error, or a
    /// wrong path from the config.
    ///
    fn launch(&self, config: &dyn Config, root_dir: &Path) -> Result<(), LaunchError> {
        let spring_path = root_dir.join(config.get_spring_relative_path());
        let write_dir_path = root_dir.join(config.get_write_dir_relative_path());
        let start_script_path = root_dir.join(config.get_start_script_relative_path());

        Command::new(spring_path.as_path())
            .env(SPRING_WRITEDIR_ENV_VAR, write_dir_path.as_path())
            .arg(start_script_path.as_path())
            .spawn()?;

        Ok(())
    }
}
