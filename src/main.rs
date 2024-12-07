use anyhow::{Context, Result};
use async_nats::{jetstream, Message};
use futures::TryStreamExt;
use internal::{
    config::config::Config,
    core::domain::{self},
    inbound::nats::Nats,
    utils::pem::PemUtils,
};
#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    PemUtils::init_provider();
    let conf = Config::load("config.toml").unwrap();
    let nats = Nats::new(conf.nats).unwrap();
    let client = nats.connect().await.unwrap();
    let context = jetstream::new(client.clone());
    let consumer = nats.create_consumer(context).await?;

    loop {
        let mut messages = consumer.messages().await?;
        while let Some(message) = messages.try_next().await? {
            println!(
                "Received message: {}",
                String::from_utf8_lossy(&message.payload)
            );
            message.ack().await?;
        }
    }
}

pub trait Convert {
    fn to_domain(&self) -> Result<domain::message::Message>;
}

impl Convert for async_nats::Message {
    fn to_domain(&self) -> Result<domain::message::Message> {
        std::str::from_utf8(self.payload.as_ref())
            .context("Cannot convert raw payload to UTF-8")
            .map(|utf8| serde_json::from_str(utf))
    }
}
