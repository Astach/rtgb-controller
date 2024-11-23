use anyhow::Result;
use async_nats::jetstream::{self, consumer};
use internal::{config::Config, inbound::nats::Nats};
#[tokio::main]
async fn main() -> Result<consumer::Consumer<consumer::pull::Config>> {
    let conf = Config::load("config.toml").unwrap();
    let nats = Nats::new(conf.nats_config).unwrap();
    let client = nats.connect().await.unwrap();
    let context = jetstream::new(client);
    nats.create_consumer(context).await;
client.subscribe(jj)
    loop {
        if let Some(msg) = sub.next().await {
            println!("Received: {}", String::from_utf8_lossy(&msg.payload));
        }
    }
}
