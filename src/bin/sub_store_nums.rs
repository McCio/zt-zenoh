use tokio::task::JoinHandle;
use zenoh::bytes::ZBytes;
use zenoh::Session;
use samp::utils::signals::{PerimeterStatus, WindowStatus};

#[tokio::main]
async fn main() {
    let config = zenoh::Config::from_json5(
        r#"
{
    plugins_loading: { enabled: true },
    plugins: {
        storage_manager: {
            storages: {
                avg: {
                    key_expr: "random/average",
                    volume: "memory",
                },
            },
        },
    },
}"#,
    )
    .unwrap();
    let session = zenoh::open(config).await.unwrap();
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
