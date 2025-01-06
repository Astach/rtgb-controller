use serde::Deserialize;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

use super::app_config::{CertConfig, CertFileType};

#[derive(Deserialize)]
pub struct PostgresConfig {
    pub database: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub cert: CertConfig,
    pub tables: Vec<String>,
}

impl PostgresConfig {
    pub fn options(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .database(&self.database)
            .host(&self.host)
            .username(&self.username)
            .ssl_mode(PgSslMode::VerifyFull)
            .ssl_root_cert(self.cert.get_path_of(CertFileType::Ca))
            .ssl_client_key(self.cert.get_path_of(CertFileType::Key))
            .ssl_client_cert(self.cert.get_path_of(CertFileType::Cert))
    }
}
