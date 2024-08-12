pub mod constants;

use config::{builder::DefaultState, ConfigBuilder, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub _app: AppConfig,
    pub paths: PathConfig,
    pub brew: BrewConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub _name: String,
    pub _version: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PathConfig {
    pub _dotfiles: PathBuf,
    pub backups: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BrewConfig {
    pub formula_file: PathBuf,
    pub cask_file: PathBuf,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let builder = ConfigBuilder::<DefaultState>::default()
            .add_source(File::with_name("config/default"))
            .add_source(Environment::with_prefix("APP"));

        builder.build()?.try_deserialize()
    }
}
