use anyhow::Result;
use futures::future::BoxFuture;

use async_nats::{
    SubscribeError, Subscriber,
    jetstream::{self, stream},
};

use crate::config::nats_config::StreamConfig;

pub struct NatsConsumer {
    stream_config: StreamConfig,
}
impl NatsConsumer {
    pub fn new(stream_config: StreamConfig) -> Result<NatsConsumer> {
        Ok(NatsConsumer {
            stream_config: stream_config,
        })
    }

    pub async fn create_consumer(
        &self,
        context: async_nats::jetstream::Context,
    ) -> Result<jetstream::consumer::Consumer<jetstream::consumer::pull::Config>> {
        context
            .create_stream(self.stream_config())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create stream: {}", e))?
            .create_consumer(self.consumer_config())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create consumer: {}", e))
    }

    fn stream_config(&self) -> jetstream::stream::Config {
        jetstream::stream::Config {
            name: self.stream_config.name.to_string(),
            subjects: self.stream_config.subjects.clone(),
            retention: stream::RetentionPolicy::WorkQueue,
            ..Default::default()
        }
    }
    fn consumer_config(&self) -> jetstream::consumer::pull::Config {
        jetstream::consumer::pull::Config {
            durable_name: Some(self.stream_config.name.to_string()),
            filter_subjects: self.stream_config.subjects.to_owned(),
            ..Default::default()
        }
    }
    pub async fn subscribe(
        &self,
        client: &async_nats::Client,
    ) -> BoxFuture<'static, Result<Subscriber, SubscribeError>> {
        let sub = self.stream_config.delivery_subject.to_string();
        let client = client.clone();
        Box::pin(async move { client.subscribe(sub).await })
            as BoxFuture<'static, Result<Subscriber, SubscribeError>>
    }
}

#[cfg(test)]
mod tests {

    use crate::config::nats_config::NatsConfig;

    use super::*;

    #[test]
    fn should_create_consumer_config() {
        let stream_config = NatsConfig::default().consumer;
        let nats = NatsConsumer::new(stream_config.clone()).unwrap();
        let consumer_config = nats.consumer_config();
        assert_eq!(consumer_config.durable_name.unwrap(), stream_config.name);
        assert_eq!(consumer_config.filter_subjects, stream_config.subjects);
    }
    #[test]
    fn should_create_stream_config() {
        let stream_config = NatsConfig::default().consumer;
        let nats = NatsConsumer::new(stream_config.clone()).unwrap();
        let stream_config = nats.stream_config();
        assert_eq!(stream_config.retention, stream::RetentionPolicy::WorkQueue);
        assert_eq!(stream_config.name, stream_config.name);
        assert_eq!(stream_config.subjects, stream_config.subjects);
    }
}
