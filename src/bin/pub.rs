use samp::utils;
use utils::publisher::FixedIntervalPublisher;
use utils::rate_producer::RateProducer;
use utils::rate::Rate;
use utils::rate::TimeUnit::Seconds;

#[tokio::main]
async fn main() {
    let session = zenoh::open(zenoh::Config::default()).await.unwrap();
    let (publisher, run_status_update) = RateProducer::new_first(
        Rate {
            events: 5,
            time_unit: Seconds,
        },
        "windows/studio",
        &session,
    ).await;
    let sleeper = tokio::task::spawn(async move {
        println!("sleeping 30s");
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        println!("stopping publisher");
        run_status_update.send_replace(false);
    });
    assert!(publisher.run().await.unwrap(), "It didn't start");
    session.close().await.unwrap();
    sleeper.await.expect("This shouldn't fail");
}
