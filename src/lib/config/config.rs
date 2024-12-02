use anyhow::Result;
use serde::Deserialize;
use std::fs::{self};
use thiserror::Error;

use super::{nats_config::NatsConfig, postgres_config::PostgresConfig};

#[derive(Deserialize)]
pub struct Config {
    pub nats: NatsConfig,
    pub postgres: PostgresConfig,
}

impl Config {
    pub fn load(file_name: &str) -> Result<Config, ConfigError> {
        let content = fs::read_to_string(file_name)?;
        Ok(toml::from_str(&content)?)
    }
}
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Could not read config file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Could not parse TOML config: {0}")]
    TomlError(#[from] toml::de::Error),
}
