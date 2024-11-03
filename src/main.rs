#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());

    let client = async_nats::connect(nats_url).await?;

    let jetstream = jetstream::new(client);

    let stream_name = String::from("EVENTS");

    let consumer: PullConsumer = jetstream
        .create_stream(jetstream::stream::Config {
            name: stream_name,
            subjects: vec!["events.>".to_string()],
            ..Default::default()
        })
        .await?
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("consumer".to_string()),
            ..Default::default()
        })
        .await?;
    Ok(())
}
