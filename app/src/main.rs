mod config;
mod inbound;
mod nats_client;
mod outbound;
mod utils;
//TODO move the mod into lib.rs so they can be used for IT tests.

use std::sync::Arc;

use anyhow::Result;
use async_nats::jetstream;
use config::app_config::AppConfig;
use futures::TryStreamExt;
use inbound::model::event::Event;
use inbound::nats::NatsConsumer;
use internal::{
    domain::message::{Message, MessageType},
    port::command::CommandExecutorDriverPort,
    port::command::CommandSchedulerDriverPort,
    service::{command_executor_service::CommandExecutorService, command_scheduler_service::CommandSchedulerService},
};
use log::{debug, error};
use nats_client::NatsClient;
use outbound::{nats_publisher::NatsPublisher, postgres::CommandRepository};
use sqlx::postgres::PgPoolOptions;
use tokio::sync::OnceCell;
use utils::pem::PemUtils;
static CMD_REPOSITORY: OnceCell<Arc<CommandRepository>> = OnceCell::const_new();

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    PemUtils::init_provider();
    let conf = AppConfig::load("config.toml").unwrap();
    let nats = NatsClient {
        client_config: conf.nats.client,
    };
    let client = nats.connect().await.unwrap();
    let consumer = NatsConsumer::new(conf.nats.consumer).unwrap();
    let context = jetstream::new(client.clone());
    let consumer = consumer.create_consumer(&context).await?;

    let cmd_repository = CMD_REPOSITORY
        .get_or_init(async || {
            let pool = PgPoolOptions::new()
                .connect_with(conf.postgres.options())
                .await
                .unwrap();
            Arc::new(CommandRepository::new(pool))
        })
        .await;
    let nats_publisher = NatsPublisher::new(client, conf.nats.publisher);
    let scheduler_service = CommandSchedulerService::new(cmd_repository.clone());
    let executor_service = CommandExecutorService::new(cmd_repository.clone(), nats_publisher);

    loop {
        let mut messages = consumer.messages().await?;
        while let Some(input) = messages.try_next().await? {
            let message = Event::try_from(&input)
                .and_then(Message::try_from)
                .inspect_err(|e| error!("{e}"))?;

            match message.message_type {
                MessageType::Schedule(schedule_message_data) => scheduler_service
                    .schedule(schedule_message_data)
                    .await
                    .inspect(|it| debug!("Command Processed, {:?} commmand(s) created", it))
                    .inspect_err(|e| error!("{e}"))
                    .map(|_| ())?,

                MessageType::Tracking(tracking_message_data) => executor_service
                    .process(tracking_message_data)
                    .await
                    .inspect(|_| debug!("Message Processed, commmand(s) executed/updated"))
                    .inspect_err(|e| error!("{e}"))?,
            }
            // {
            //        Ok(fut) => match fut.await {
            //            Ok(x) => debug!("Message Processed, {:?} commmand(s) created", x),
            //            Err(e) => error!("Unable to process event {:?}", e),
            //        },
            //        Err(e) => {
            //            error!("Unable to convert event into a domain message {:?}", e)
            //        }
            //    }

            input.ack().await.map_err(|e| anyhow::anyhow!(e))?;
        }
    }
}
