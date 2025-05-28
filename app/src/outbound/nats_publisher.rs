use async_nats::Client;
use internal::port::publisher::{HardwareAction, PublisherDrivenPort};

use crate::config::nats_config::PublisherConfig;

pub struct NatsPublisher {
    client: Client,
    publisher_config: PublisherConfig,
}
impl NatsPublisher {
    pub fn new(client: Client, publisher_config: PublisherConfig) -> Self {
        NatsPublisher {
            publisher_config,
            client,
        }
    }
}
//TODO V2 should have different publisher to support different device types (Shelly models,
//phillips ...)
impl PublisherDrivenPort for NatsPublisher {
    async fn publish(&self, action: HardwareAction) -> anyhow::Result<()> {
        self.client
            .publish(
                Self::build_topic(
                    &self.publisher_config.command_topic_template,
                    "shellyplug-s",
                    action.get_hardware_id().as_str(),
                ),
                match action {
                    HardwareAction::START(_) => "on".into(),
                    HardwareAction::STOP(_) => "off".into(),
                },
            )
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }
}

impl NatsPublisher {
    fn build_topic(template: &str, model: &str, deviceid: &str) -> String {
        template.replace("{model}", model).replace("{deviceid}", deviceid)
    }
}
