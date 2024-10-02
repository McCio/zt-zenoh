use samp::utils::signals::Status;

#[tokio::main]
async fn main() {
    let session = zenoh::open(zenoh::Config::default()).await.unwrap();
    let key = session.declare_keyexpr("windows/*").await.unwrap();
    let subscriber = session.declare_subscriber(key).await.unwrap();
    while let Ok(sample) = subscriber.recv_async().await {
        println!("Received on {}: {}", sample.key_expr(), match sample.payload().deserialize::<Status>() {
            Ok(x) => x.into(),
            Err(_) => "<Invalid value>",
        });
    }
    session.close().await.unwrap();
}
