use serde::Deserialize;

use super::config::CertConfig;

#[derive(Deserialize)]
pub struct NatsConfig {
    pub host: String,
    pub port: u16,
    pub cert: CertConfig,
    pub consumer: ConsumerConfig,
}

#[derive(Deserialize)]
pub struct ConsumerConfig {
    pub subjects: Vec<String>,
    pub delivery_subject: String,
    pub name: String,
}
