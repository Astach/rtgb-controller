use anyhow::{Context, Result};
use serde::Deserialize;
use std::fmt::Write;

#[derive(Deserialize)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
    pub ssl_mode: String,
}

impl PostgresConfig {
    pub fn to_url(&self) -> Result<String> {
        let mut url = String::new();
        write!(
            &mut url,
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            self.user, self.password, self.host, self.port, self.name, self.ssl_mode
        )
        .context("Unable to build postgres url")
        .map(|_| url)
    }
}
