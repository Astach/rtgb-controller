use anyhow::{Context, Result};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, pkcs8_private_keys};

pub struct PemUtils {}

impl PemUtils {
    pub fn parse_certificate(data: Vec<u8>) -> Result<CertificateDer<'static>> {
        Ok(certs(&mut &data[..])
            .find_map(|cert_res| cert_res.ok())
            .context("Failed to parse certificate")?)
    }
    pub fn parse_private_key(data: Vec<u8>) -> Result<PrivateKeyDer<'static>> {
        Ok(pkcs8_private_keys(&mut &data[..])
            .find_map(|key_result| key_result.ok()) // Find the first successful key parse
            .map(PrivateKeyDer::Pkcs8)
            .context("Failed to parse any valid private key")?)
    }
}
