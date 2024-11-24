use async_nats::jetstream;
use futures::StreamExt;
use internal::{config::Config, inbound::nats::Nats, utils::pem::PemUtils};
#[tokio::main]
async fn main() {
    PemUtils::init_provider();
    let conf = Config::load("config.toml").unwrap();
    let nats = Nats::new(conf.nats).unwrap();
    let client = nats.connect().await.unwrap();
    let context = jetstream::new(client.clone());
    nats.create_consumer(context).await.unwrap();

    let subs = nats.subscribe(&client).await;
    let results = futures::future::join_all(subs).await;
    for subscription in results {
        match subscription {
            Ok(s) => process_messages(s).await,
            Err(e) => {
                eprint!("Shit happened {:?}", e)
            }
        }
    }
    async fn process_messages(mut subscriber: async_nats::Subscriber) {
        // Process each message
        while let Some(message) = subscriber.next().await {
            println!("Received message: {:?}", message);
        }
    }
}
