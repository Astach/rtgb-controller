use testcontainers::{ImageExt, core::Mount};
use testcontainers_modules::nats::Nats;

struct NatsHelper;
impl NatsHelper {
    async fn start_nats() {
        // --- Setup NATS (TLS, no auth) ---
        let nats_image = Nats::default()
            // Mount NATS config and certs into expected container locations
            .with_mount(Mount::bind_mount(
                "./docker/nats/server.template.conf",
                "/nats/server.conf",
            ))
            .with_mount(Mount::bind_mount("../../docker/nats/ca.crt", "/certs/ca.crt"))
            .with_mount(Mount::bind_mount("../../docker/nats/server.crt", "/certs/server.crt"))
            .with_mount(Mount::bind_mount("../../docker/nats/server.key", "/certs/server.key"));
    }
}
