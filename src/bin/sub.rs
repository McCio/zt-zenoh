use samp::utils::signals::{PerimeterStatus, WindowStatus};
use tokio::task::JoinHandle;
use zenoh::bytes::ZBytes;
use zenoh::Session;

#[tokio::main]
async fn main() {
    let session = zenoh::open(zenoh::Config::default()).await.unwrap();
    let windows = hear_for_windows(&session).await;
    let perimeters = hear_for_perimeters(&session).await;
    windows.await.unwrap();
    perimeters.await.unwrap();
    session.close().await.unwrap();
}

async fn hear_for_windows(session: &Session) -> JoinHandle<()> {
    let key = session.declare_keyexpr("window/*").await.unwrap();
    let subscriber = session.declare_subscriber(key).await.unwrap();
    tokio::task::spawn(async move {
        while let Ok(sample) = subscriber.recv_async().await {
            println!(
                "Received on {}: {}",
                sample.key_expr(),
                <&ZBytes as TryInto<WindowStatus>>::try_into(sample.payload()).map_or_else(|_| "<Invalid value>", |x| x.into())
            );
        }
    })
}

async fn hear_for_perimeters(session: &Session) -> JoinHandle<()> {
    let key = session.declare_keyexpr("perimeter/*").await.unwrap();
    let subscriber = session.declare_subscriber(key).await.unwrap();
    tokio::task::spawn(async move {
        while let Ok(sample) = subscriber.recv_async().await {
            println!(
                "Received on {}: {}",
                sample.key_expr(),
                <&ZBytes as TryInto<PerimeterStatus>>::try_into(sample.payload()).map_or_else(|_| "<Invalid value>", |x| x.into())
            );
        }
    })
}
