use anyhow::Result;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use serde::Deserialize;
use std::fs::{self};
use thiserror::Error;

use crate::utils::{file::FileUtils, pem::PemUtils};

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

#[derive(Deserialize)]
pub struct CertConfig {
    absolute_folder_path: String,
    key_file_name: String,
    cert_file_name: String,
    root_ca_file_name: String,
}

pub enum CertFileType {
    Key,
    Cert,
    Ca,
}

impl CertConfig {
    pub fn get_path_of(&self, cert_type: CertFileType) -> String {
        match cert_type {
            CertFileType::Ca => format!("{}/{}", self.absolute_folder_path, self.root_ca_file_name),
            CertFileType::Cert => format!("{}/{}", self.absolute_folder_path, self.cert_file_name),
            CertFileType::Key => format!("{}/{}", self.absolute_folder_path, self.key_file_name),
        }
    }
    pub fn private_key(&self) -> Result<PrivateKeyDer<'static>> {
        let key_path = self.get_path_of(CertFileType::Key);
        let key_data = FileUtils::load(&key_path).unwrap();
        PemUtils::parse_private_key(key_data)
    }

    pub fn certificate(&self) -> Result<CertificateDer<'static>> {
        let cert_path = self.get_path_of(CertFileType::Cert);
        let cert_data = FileUtils::load(&cert_path).unwrap();
        PemUtils::parse_certificate(cert_data)
    }
    pub fn root_ca(&self) -> Result<CertificateDer<'static>> {
        let ca_path = self.get_path_of(CertFileType::Ca);
        let ca_data = FileUtils::load(&ca_path).unwrap();
        PemUtils::parse_certificate(ca_data)
    }
}
