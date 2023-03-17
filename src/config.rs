use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::Deserialize;
use std::result::Result;
use thiserror::Error;

const CONFIG_FILENAME: &str = "config.toml";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config error")]
    BuildError(#[from] figment::Error),
}

pub trait Config {
    fn get_spring_relative_path(&self) -> &str;
    fn get_start_script_relative_path(&self) -> &str;
    fn get_write_dir_relative_path(&self) -> &str;
}

#[derive(Deserialize)]
pub struct AutohostConfig {
    spring_relative_path: String,
    start_script_relative_path: String,
    write_dir_relative_path: String,
}

/// The `AutohostConfig` uses the [figment crate](https://docs.rs/figment/latest/figment/)
/// To deserialize configuration data from the `config.toml` file to
/// be used by the autohost. Env vars can also be used with a few minor changes.
impl AutohostConfig {
    pub fn build() -> Result<Self, ConfigError> {
        Ok(Figment::new()
            .merge(Toml::file(CONFIG_FILENAME))
            .extract()?)
    }
}

impl Config for AutohostConfig {
    fn get_spring_relative_path(&self) -> &str {
        &self.spring_relative_path
    }

    fn get_start_script_relative_path(&self) -> &str {
        &self.start_script_relative_path
    }

    fn get_write_dir_relative_path(&self) -> &str {
        &self.write_dir_relative_path
    }
}
