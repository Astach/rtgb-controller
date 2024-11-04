/*
 let client = async_nats::connect_with_options(
    "tls://localhost:4222",
    ConnectOptions::new()
        .tls_client_cert(
            Path::new("certs/client.crt"),
            Path::new("certs/client.key"),
        )
        .add_root_certificate(Path::new("certs/ca.crt"))
).await?;
*/
