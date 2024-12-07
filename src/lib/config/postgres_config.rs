use serde::Deserialize;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

use super::config::{CertConfig, CertFileType};

#[derive(Deserialize)]
pub struct PostgresConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub cert: CertConfig,
}

impl PostgresConfig {
    pub fn options(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(self.host.as_str())
            .ssl_mode(PgSslMode::VerifyFull)
            .ssl_root_cert(self.cert.get_path_of(CertFileType::Ca))
            .ssl_client_key(self.cert.get_path_of(CertFileType::Key))
            .ssl_client_cert(self.cert.get_path_of(CertFileType::Cert))
    }
}
