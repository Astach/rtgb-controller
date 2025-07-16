use serde::Deserialize;

use super::app_config::CertConfig;

#[derive(Deserialize, Default, Clone)]
pub struct NatsConfig {
    pub client: ClientConfig,
    pub consumer: ConsumerConfig,
    pub publisher: PublisherConfig,
}

#[derive(Deserialize, Default, Clone)]
pub struct ConsumerConfig {
    pub subjects: Vec<String>,
    pub delivery_subject: String,
    pub name: String,
}

#[derive(Deserialize, Default, Clone)]
//https://shelly-api-docs.shelly.cloud/gen1/#shelly-plug-plugs-mqtt
pub struct PublisherConfig {
    pub command_topic_template: String,
}

#[derive(Deserialize, Default, Clone)]
pub struct ClientConfig {
    pub host: String,
    pub port: u16,
    pub cert: CertConfig,
    pub creds_path: String,
}
