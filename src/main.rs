use async_nats::jetstream;
use futures::TryStreamExt;
use internal::{config::Config, inbound::nats::Nats, utils::pem::PemUtils};
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
    //     let subs = nats.subscribe(&client).await;
    //     match subs.await {
    //         Ok(s) => process_messages(s).await,
    //         Err(e) => {
    //             eprint!("Shit happened {:?}", e)
    //         }
    //     }
    //
    //     async fn process_messages(mut subscriber: async_nats::Subscriber) {
    //         // Process each message
    //         while let Some(message) = subscriber.next().await {
    //             println!("Received message: {:?}", message);
    //             subscriber.ack(message);
    //         }
    //     }
}
