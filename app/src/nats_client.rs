use crate::config::{app_config::CertificateProvider, nats_config::ClientConfig as NatsClientConf};
use anyhow::{Context, Result};
use async_nats::ConnectOptions;
use rustls::{ClientConfig, RootCertStore};

pub struct NatsClient {
    pub client_config: NatsClientConf,
}
impl NatsClient {
    fn client_configuration(certificate_provider: &impl CertificateProvider) -> Result<ClientConfig> {
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
        let address = format!("tls://{}:{}", self.client_config.host, self.client_config.port);
        let options = ConnectOptions::new()
            .tls_client_config(NatsClient::client_configuration(&self.client_config.cert).unwrap())
            .require_tls(true)
            .name("rtgb-controller");

        async_nats::connect_with_options(address, options)
            .await
            .context("Cannot connect to nats server")
    }
}

#[cfg(test)]
mod test {
    use super::NatsClient;
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
        let root_certified_keys = generate_simple_self_signed(vec![subject_name.to_string()]).unwrap();

        // Generate client certificate
        let subject_name = "Client Certificate";
        let client_certified_keys = generate_simple_self_signed(vec![subject_name.to_string()]).unwrap();
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

        let result = NatsClient::client_configuration(&cert_config);
        result.unwrap();
    }

    #[test]
    #[should_panic]
    fn should_panic_if_invalid_certs() {
        let mut cert_config = MockCertConfig::new();
        cert_config
            .expect_root_ca()
            .return_once(move || Err(Error::msg("Cert error")));
        let result = NatsClient::client_configuration(&cert_config);
        result.unwrap_err();
    }
}
