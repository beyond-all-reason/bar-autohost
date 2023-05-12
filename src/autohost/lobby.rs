use std::result::Result;

use thiserror::Error;

use super::spring::LaunchError;
use super::spring::Spring;
use crate::server_coms::server_error::ServerError;
use crate::utils::config::{Config, ConfigError};
use crate::utils::environment::{Environment, EnvironmentError};

#[derive(Error, Debug)]
pub enum LobbyError {
    #[error("Spring error")]
    Spring(#[from] LaunchError),
    #[error("Environment error")]
    Environment(#[from] EnvironmentError),
    #[error("Config error")]
    Config(#[from] ConfigError),
    #[error("Server error")]
    Server(#[from] ServerError),
}

pub struct Lobby<'a> {
    config: &'a dyn Config,
    spring: &'a dyn Spring,
    environment: &'a dyn Environment,
}

impl<'a> Lobby<'_> {
    pub fn new(
        config: &'a dyn Config,
        spring: &'a dyn Spring,
        environment: &'a dyn Environment,
    ) -> Lobby<'a> {
        Lobby {
            config,
            spring,
            environment,
        }
    }

    pub fn start_game(&self) -> Result<(), LobbyError> {
        let root_dir = self.environment.get_current_dir()?;

        Ok(self.spring.launch(self.config, &root_dir)?)
    }
}
