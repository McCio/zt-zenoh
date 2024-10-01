#[tokio::main]
async fn main() {
    let session = zenoh::open(zenoh::Config::default()).await.unwrap();
    let key = session.declare_keyexpr("windows/studio").await.unwrap();
    let subscriber = session.declare_subscriber(key).await.unwrap();
    while let Ok(sample) = subscriber.recv_async().await {
        println!("Received: {}", sample.payload().deserialize::<String>().unwrap());
    }
    session.close().await.unwrap();
}
