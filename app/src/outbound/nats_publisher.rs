use internal::{domain::command::Command, port::publisher::PublisherDrivenPort};

use crate::config::nats_config::StreamConfig;

pub struct NatsPublisher {
    publisher_config: StreamConfig,
}
impl NatsPublisher {
    pub fn new(publisher_config: StreamConfig) -> Self {
        NatsPublisher { publisher_config }
    }
}

impl PublisherDrivenPort for NatsPublisher {
    async fn publish(&self, command: &Command) -> anyhow::Result<()> {
        todo!()
    }
}
