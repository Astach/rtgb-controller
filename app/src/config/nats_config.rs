use serde::Deserialize;

use super::app_config::CertConfig;

#[derive(Deserialize, Default, Clone)]
pub struct NatsConfig {
    pub host: String,
    pub port: u16,
    pub cert: CertConfig,
    pub consumer: ConsumerConfig,
}

#[derive(Deserialize, Default, Clone)]
pub struct ConsumerConfig {
    pub subjects: Vec<String>,
    pub delivery_subject: String,
    pub name: String,
}
