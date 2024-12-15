use anyhow::Result;
use async_nats::jetstream;
use futures::{StreamExt, TryStreamExt};
use internal::{
    config::config::Config,
    core::{port::messaging::MessageDriverPort, service::message_service::MessageService},
    inbound::{model::event::Event, nats::Nats},
    outbound::postgres::MessageRepository,
    utils::pem::PemUtils,
};
use log::{debug, error};
use sqlx::postgres::PgPoolOptions;
#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    env_logger::init();
    PemUtils::init_provider();
    let conf = Config::load("config.toml").unwrap();
    let nats = Nats::new(conf.nats).unwrap();
    let client = nats.connect().await.unwrap();
    let context = jetstream::new(client.clone());
    let consumer = nats.create_consumer(context).await?;

    let pool = PgPoolOptions::new()
        .connect_with(conf.postgres.options())
        .await?;
    let message_repository = MessageRepository::new(&pool);
    let message_service = MessageService::new(message_repository);

    loop {
        let mut messages = consumer.messages().await?;
        while let Some(message) = messages.try_next().await? {
            match Event::try_from(&message)
                .map(|event| event.to_domain())?
                .map(|msg| message_service.process(msg))
            {
                Ok(_) => {
                    debug!("Processed incoming nats msg")
                }
                Err(e) => {
                    error!("Unable to process event {:?}", e)
                }
            }

            message.ack().await?;
        }
    }
}
