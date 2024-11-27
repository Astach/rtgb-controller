use anyhow::{Context, Result};
use futures::future::BoxFuture;
use rustls::{ClientConfig, RootCertStore};

use crate::config::NatsConfig;
use async_nats::{
    jetstream::{self, stream},
    ConnectOptions, SubscribeError, Subscriber,
};

pub struct Nats {
    nats_config: NatsConfig,
}

impl Nats {
    pub fn new(nats_config: NatsConfig) -> Result<Nats> {
        Ok(Nats { nats_config })
    }
    fn client_configuration(&self) -> Result<ClientConfig> {
        let mut store = RootCertStore::empty();
        let ca = self.nats_config.cert.root_ca().unwrap();
        let cert = self.nats_config.cert.certificate().unwrap();
        let private_key = self.nats_config.cert.private_key().unwrap();
        store.add(ca).unwrap();
        ClientConfig::builder()
            .with_root_certificates(store) // Add the CA certs
            .with_client_auth_cert(vec![cert], private_key) // Set the client cert and private key
            .context("Unable to build client configuration!")
    }

    pub async fn connect(&self) -> Result<async_nats::Client> {
        let address = format!("tls://{}:{}", self.nats_config.host, self.nats_config.port);
        let options = ConnectOptions::new()
            .tls_client_config(self.client_configuration().unwrap())
            .require_tls(true)
            .name("rtgb-controller");

        async_nats::connect_with_options(address, options)
            .await
            .context("Cannot connect to nats server")
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
            name: self.nats_config.consumer.name.to_string(),
            subjects: self.nats_config.consumer.subjects.clone(),
            retention: stream::RetentionPolicy::WorkQueue,
            ..Default::default()
        }
    }
    fn consumer_config(&self) -> jetstream::consumer::pull::Config {
        jetstream::consumer::pull::Config {
            durable_name: Some(self.nats_config.consumer.name.to_string()),
            filter_subjects: self.nats_config.consumer.subjects.to_owned(),
            ..Default::default()
        }
    }
    pub async fn subscribe(
        &self,
        client: &async_nats::Client,
    ) -> BoxFuture<'static, Result<Subscriber, SubscribeError>> {
        let sub = self.nats_config.consumer.delivery_subject.to_string();
        let client = client.clone();
        Box::pin(async move { client.subscribe(sub).await })
            as BoxFuture<'static, Result<Subscriber, SubscribeError>>
    }
}
