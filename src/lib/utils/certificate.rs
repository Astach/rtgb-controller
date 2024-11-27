use rustls_pemfile::certs;

struct PemUtils {}

impl PemUtils {
    pub fn parse(data: Vec<u8>) {
        certs(&mut &data[..])
            .find_map(|cert_res| cert_res.ok())
            .context("Failed to parse certificate")?
    }
}
