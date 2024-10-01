use async_std::task;
use rate::TimeUnit::Seconds;

#[path = "../lib/rate.rs"]
mod rate;

#[tokio::main]
async fn main() {
    let session = zenoh::open(zenoh::Config::default()).await.unwrap();
    let interval = rate::events_per_unit(5, Seconds);
    println!("Will publish every {:?}", interval);
    let key = session.declare_keyexpr("windows/studio").await.unwrap();
    let key_publisher = session.declare_publisher(key).await.unwrap();
    while key_publisher.put("closed").await.is_ok() {
        println!("published");
        task::sleep(interval).await
    }
    session.close().await.unwrap();
}
