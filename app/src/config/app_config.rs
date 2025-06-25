use anyhow::{Result, anyhow};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use serde::Deserialize;
use std::{
    fs::{self},
    path::Path,
};

use crate::utils::{file::FileUtils, pem::PemUtils};

use super::{nats_config::NatsConfig, postgres_config::PostgresConfig};

#[derive(Deserialize)]
pub struct AppConfig {
    pub nats: NatsConfig,
    pub postgres: PostgresConfig,
}

impl AppConfig {
    pub fn load(file_name: &str) -> anyhow::Result<AppConfig> {
        let project_root = env!("CARGO_MANIFEST_DIR");
        let file_path = Path::new(project_root).join(file_name);
        let content = fs::read_to_string(file_path).map_err(|err| anyhow!("Could not read config file: {:?}", err))?;
        toml::from_str(&content).map_err(|err| anyhow!("Could not parse TOML config: {:?}", err))
    }
}

#[derive(Deserialize, Default, Clone)]
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

pub trait CertificateProvider {
    fn get_path_of(&self, cert_type: CertFileType) -> String;
    fn private_key(&self) -> Result<PrivateKeyDer<'static>>;
    fn certificate(&self) -> Result<CertificateDer<'static>>;
    fn root_ca(&self) -> Result<CertificateDer<'static>>;
}

#[cfg_attr(test, mockall::automock)]
impl CertificateProvider for CertConfig {
    fn get_path_of(&self, cert_type: CertFileType) -> String {
        match cert_type {
            CertFileType::Ca => format!("{}/{}", self.absolute_folder_path, self.root_ca_file_name),
            CertFileType::Cert => format!("{}/{}", self.absolute_folder_path, self.cert_file_name),
            CertFileType::Key => format!("{}/{}", self.absolute_folder_path, self.key_file_name),
        }
    }
    fn private_key(&self) -> Result<PrivateKeyDer<'static>> {
        let key_path = self.get_path_of(CertFileType::Key);
        let key_data = FileUtils::load(&key_path).unwrap();
        PemUtils::parse_private_key(key_data)
    }

    fn certificate(&self) -> Result<CertificateDer<'static>> {
        let cert_path = self.get_path_of(CertFileType::Cert);
        let cert_data = FileUtils::load(&cert_path).unwrap();
        PemUtils::parse_certificate(cert_data)
    }

    fn root_ca(&self) -> Result<CertificateDer<'static>> {
        let ca_path = self.get_path_of(CertFileType::Ca);
        let ca_data = FileUtils::load(&ca_path).unwrap();
        PemUtils::parse_certificate(ca_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_load_app_config() {
        AppConfig::load("config.toml").unwrap();
    }

    #[test]
    fn should_return_correct_cert_file_path() {
        let cert_conf = CertConfig {
            absolute_folder_path: String::from("path"),
            key_file_name: String::from("key"),
            cert_file_name: String::from("cert"),
            root_ca_file_name: String::from("ca"),
        };
        assert_eq!(cert_conf.get_path_of(CertFileType::Cert), "path/cert");
        assert_eq!(cert_conf.get_path_of(CertFileType::Key), "path/key");
        assert_eq!(cert_conf.get_path_of(CertFileType::Ca), "path/ca");
    }
}
