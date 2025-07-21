use testcontainers::{ImageExt, runners::AsyncRunner};
use testcontainers_modules::nats::{Nats, NatsServerCmd};

#[test]
fn should_create_command_schedule_in_db() {
    let nats_image = Nats::default().with_cmd(vec![
        "--tls".to_string(),
        "--tlscert=./certs/server.crt".to_string(),
        "--tlskey=./certs/server.key".to_string(),
        "--tlsverify".to_string(),
        "--tlscacert=./certs/ca.pem".to_string(),
    ]);
}
