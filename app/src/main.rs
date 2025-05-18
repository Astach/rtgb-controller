mod config;
mod inbound;
mod nats_client;
mod outbound;
mod utils;
//TODO move the mod into lib.rs so they can be used for IT tests.

use anyhow::Result;
use async_nats::jetstream;
use config::app_config::{self, AppConfig};
use futures::TryStreamExt;
use inbound::model::event::Event;
use inbound::nats::NatsConsumer;
use internal::{
    domain::message::Message, port::messaging::MessageDriverPort,
    service::message_service::MessageService,
};
use log::{debug, error};
use nats_client::NatsClient;
use outbound::{nats_publisher::NatsPublisher, postgres::MessageRepository};
use sqlx::postgres::PgPoolOptions;
use utils::pem::PemUtils;
#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    env_logger::init();
    PemUtils::init_provider();
    let conf = AppConfig::load("config.toml").unwrap();
    let nats = NatsClient {
        client_config: conf.nats.client,
    };
    let client = nats.connect().await.unwrap();
    let consumer = NatsConsumer::new(conf.nats.consumer).unwrap();
    let context = jetstream::new(client.clone());
    let consumer = consumer.create_consumer(context).await?;

    let pool = PgPoolOptions::new()
        .connect_with(conf.postgres.options())
        .await?;
    let message_repository = MessageRepository::new(&pool);
    let nats_publisher = NatsPublisher::new(conf.nats.publisher);
    let message_service = MessageService::new(message_repository, nats_publisher);

    loop {
        let mut messages = consumer.messages().await?;
        while let Some(message) = messages.try_next().await? {
            match Event::try_from(&message)
                .and_then(Message::try_from)
                .map(|msg| message_service.process(msg))
            {
                Ok(fut) => match fut.await {
                    Ok(x) => debug!("Message Processed, {:?} commmand(s) created", x),
                    Err(e) => error!("Unable to process event {:?}", e),
                },
                Err(e) => {
                    error!("Unable to convert event into a domain message {:?}", e)
                }
            }

            message.ack().await?;
        }
    }
}
