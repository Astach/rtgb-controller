use serde::Deserialize;

use super::app_config::CertConfig;

#[derive(Deserialize, Default, Clone)]
pub struct NatsConfig {
    pub client: ClientConfig,
    pub consumer: StreamConfig,
    pub publisher: StreamConfig,
}

#[derive(Deserialize, Default, Clone)]
pub struct StreamConfig {
    pub subjects: Vec<String>,
    pub delivery_subject: String,
    pub name: String,
}

#[derive(Deserialize, Default, Clone)]
pub struct ClientConfig {
    pub host: String,
    pub port: u16,
    pub cert: CertConfig,
}
