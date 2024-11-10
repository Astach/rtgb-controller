use serde::Deserialize;
use std::fs;
use thiserror::Error;

#[derive(Deserialize)]
pub struct Config {
    pub nats_config: NatsConfig,
}

#[derive(Deserialize)]
pub struct NatsConfig {
    pub host: String,
    pub port: i8,
    pub cert: CertConfig,
    pub consumer: ConsumerConfig,
}

#[derive(Deserialize)]
pub struct ConsumerConfig {
    pub subjects: Vec<String>,
    pub name: String,
}

#[derive(Deserialize)]
pub struct CertConfig {
    pub absolute_folder_path: String,
    pub key_file_name: String,
    pub cert_file_name: String,
    pub root_ca_file_name: String,
}

impl Config {
    pub fn load(file_name: &str) -> Result<Config, ConfigError> {
        let content = fs::read_to_string(file_name)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Could not read config file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Could not parse TOML config: {0}")]
    TomlError(#[from] toml::de::Error),
}
