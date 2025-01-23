use anyhow::{Context, Result};
use futures::future::BoxFuture;
use rustls::{ClientConfig, RootCertStore};

use async_nats::{
    jetstream::{self, stream},
    ConnectOptions, SubscribeError, Subscriber,
};

use crate::config::{app_config::CertificateProvider, nats_config::NatsConfig};

pub struct Nats {
    nats_config: NatsConfig,
}

impl Nats {
    pub fn new(nats_config: NatsConfig) -> Result<Nats> {
        Ok(Nats { nats_config })
    }

    fn client_configuration(
        certificate_provider: &impl CertificateProvider,
    ) -> Result<ClientConfig> {
        let mut store = RootCertStore::empty();
        let ca = certificate_provider.root_ca().unwrap();
        let cert = certificate_provider.certificate().unwrap();
        let private_key = certificate_provider.private_key().unwrap();
        store.add(ca).unwrap();
        ClientConfig::builder()
            .with_root_certificates(store) // Add the CA certs
            .with_client_auth_cert(vec![cert], private_key) // Set the client cert and private key
            .context("Unable to build client configuration!")
    }

    pub async fn connect(&self) -> Result<async_nats::Client> {
        let address = format!("tls://{}:{}", self.nats_config.host, self.nats_config.port);
        let options = ConnectOptions::new()
            .tls_client_config(Nats::client_configuration(&self.nats_config.cert).unwrap())
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

#[cfg(test)]
mod tests {

    use std::vec;

    use super::*;
    use crate::config::app_config::MockCertConfig;
    use anyhow::Error;
    use rcgen::generate_simple_self_signed;
    use rustls::pki_types::PrivateKeyDer;

    #[test]
    fn should_create_client_config() {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install rustls crypto provider");

        let mut cert_config = MockCertConfig::new();
        // Generate root CA
        let subject_name = "Root CA";
        let root_certified_keys =
            generate_simple_self_signed(vec![subject_name.to_string()]).unwrap();

        // Generate client certificate
        let subject_name = "Client Certificate";
        let client_certified_keys =
            generate_simple_self_signed(vec![subject_name.to_string()]).unwrap();
        let pk = client_certified_keys.key_pair.serialized_der().to_owned();
        cert_config
            .expect_root_ca()
            .return_once(move || Ok(root_certified_keys.cert.der().clone()));
        cert_config
            .expect_certificate()
            .return_once(move || Ok(client_certified_keys.cert.der().clone()));
        cert_config
            .expect_private_key()
            .return_once(move || Ok(PrivateKeyDer::try_from(pk).unwrap()));

        let result = Nats::client_configuration(&cert_config);
        result.unwrap();
    }

    #[test]
    #[should_panic]
    fn should_panic_if_invalid_certs() {
        let mut cert_config = MockCertConfig::new();
        cert_config
            .expect_root_ca()
            .return_once(move || Err(Error::msg("Cert error")));
        let result = Nats::client_configuration(&cert_config);
        result.unwrap_err();
    }
    #[test]
    fn should_create_consumer_config() {
        let nats_config = NatsConfig::default();
        let nats = Nats::new(nats_config.clone()).unwrap();
        let consumer_config = nats.consumer_config();
        assert_eq!(
            consumer_config.durable_name.unwrap(),
            nats_config.consumer.name
        );
        assert_eq!(
            consumer_config.filter_subjects,
            nats_config.consumer.subjects
        );
    }
    #[test]
    fn should_create_stream_config() {
        let nats_config = NatsConfig::default();
        let nats = Nats::new(nats_config.clone()).unwrap();
        let stream_config = nats.stream_config();
        assert_eq!(stream_config.retention, stream::RetentionPolicy::WorkQueue);
        assert_eq!(stream_config.name, nats_config.consumer.name);
        assert_eq!(stream_config.subjects, nats_config.consumer.subjects);
    }
}
