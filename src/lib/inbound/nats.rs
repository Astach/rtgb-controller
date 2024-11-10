use anyhow::{Context, Result};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ClientConfig, RootCertStore};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::{BufReader, Read};

use crate::config::NatsConfig;
use async_nats::ConnectOptions;

/*
 let client = async_nats::connect_with_options(
    "tls://localhost:4222",
    ConnectOptions::new()
        .tls_client_cert(
            Path::new("certs/client.crt"),
            Path::new("certs/client.key"),
        )
        .add_root_certificate(Path::new("certs/ca.crt"))
).await?;
*/
pub struct Nats {
    nats_config: NatsConfig,
}

impl Nats {
    fn options(&self) -> Result<ClientConfig> {
        let key_path = format!(
            "{}/{}",
            self.nats_config.cert.absolute_folder_path, self.nats_config.cert.key_file_name
        );
        let cert_path = format!(
            "{}/{}",
            self.nats_config.cert.absolute_folder_path, self.nats_config.cert.cert_file_name
        );
        let ca_path = format!(
            "{}/{}",
            self.nats_config.cert.absolute_folder_path, self.nats_config.cert.root_ca_file_name
        );
        let key_data = Nats::load(&key_path).unwrap();
        let cert_data = Nats::load(&cert_path).unwrap();
        let ca_data = Nats::load(&ca_path).unwrap();
        let (cert, key, ca) = Nats::parse(key_data, cert_data, ca_data).unwrap();
        let mut store = RootCertStore::empty();
        store.add(ca);
        ClientConfig::builder()
            .with_root_certificates(store) // Add the CA certs
            .with_client_auth_cert(vec![cert], key) // Set the client cert and private key
            .context("Chaud!")
    }
    fn load(path: &str) -> Result<Vec<u8>> {
        let mut cert_file =
            File::open(path).with_context(|| format!("Failed to open file: {}", path))?;
        let mut data = Vec::new();
        cert_file
            .read_to_end(&mut data)
            .map_err(|e| anyhow::anyhow!("Failed to read file: {}: {}", path, e))?;
        Ok(data)
    }
    fn parse(
        cert_data: Vec<u8>,
        key_data: Vec<u8>,
        ca_data: Vec<u8>,
    ) -> Result<(
        CertificateDer<'static>,
        PrivateKeyDer<'static>,
        CertificateDer<'static>,
    )> {
        // Parse the certificate chain
        let cert = certs(&mut &cert_data[..])
            .find_map(|cert_res| cert_res.ok())
            .context("Failed to parse certificate")?;

        // Collect the first valid key or return an error if none are found
        let private_key = pkcs8_private_keys(&mut &key_data[..])
            .find_map(|key_result| key_result.ok()) // Find the first successful key parse
            .map(PrivateKeyDer::Pkcs8)
            .context("Failed to parse any valid private key")?;
        let ca = certs(&mut &ca_data[..])
            .find_map(|cert_res| cert_res.ok())
            .context("Failed to parse ca")?;

        Ok((cert, private_key, ca))
    }

    async fn connect(&self) -> Result<async_nats::Client> {
        let address = format!("tls://{}:{}", self.nats_config.host, self.nats_config.port);
        let options = ConnectOptions::new()
            .tls_client_config(self.options().unwrap())
            .require_tls(true)
            // Set connection name (optional)
            .name("rtgb-controller");

        async_nats::connect_with_options(address, options)
            .await
            .context("Cannot connect to nats server")
    }
    fn subscribe() {}
}
