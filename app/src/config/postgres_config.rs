use super::app_config::CertificateProvider;
use serde::Deserialize;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

use super::app_config::{CertConfig, CertFileType};

#[derive(Deserialize, Default)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_pg_options_correctly() {
        let conf = PostgresConfig::default();
        let conf_options = conf.options();
        let options = PgConnectOptions::new()
            .database(conf.database.as_str())
            .host(conf.host.as_str())
            .username(conf.username.as_str())
            .ssl_mode(PgSslMode::VerifyFull)
            .ssl_root_cert(conf.cert.get_path_of(CertFileType::Ca))
            .ssl_client_key(conf.cert.get_path_of(CertFileType::Key))
            .ssl_client_cert(conf.cert.get_path_of(CertFileType::Cert));
        assert_eq!(conf_options.get_database(), options.get_database());
        assert_eq!(conf_options.get_host(), options.get_host());
        assert_eq!(conf_options.get_username(), options.get_username());
    }
}
