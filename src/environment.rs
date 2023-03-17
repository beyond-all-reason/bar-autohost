use std::env;
use std::path::PathBuf;
use std::result::Result;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EnvironmentError {
    #[error("Envinronment error")]
    EnvarRetrievalFailure(#[from] std::io::Error),
}

pub trait Environment {
    fn get_current_dir(&self) -> Result<PathBuf, EnvironmentError>;
}

pub struct AutohostEnvironment {}

impl AutohostEnvironment {
    pub fn new() -> AutohostEnvironment {
        AutohostEnvironment {}
    }
}

impl Environment for AutohostEnvironment {
    fn get_current_dir(&self) -> Result<PathBuf, EnvironmentError> {
        Ok(env::current_dir()?)
    }
}
