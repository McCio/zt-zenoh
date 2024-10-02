use std::time::Duration;
use tokio::sync::watch::{Receiver, Sender};
use crate::rate::Rate;
use rate::TimeUnit::Seconds;
use tokio::task;
use zenoh::bytes::ZBytes;
use zenoh::Wait;
use crate::publisher::FixedIntervalPublisher;

#[path = "../lib/publisher.rs"]
mod publisher;
#[path = "../lib/rate.rs"]
mod rate;

#[derive(strum_macros::IntoStaticStr)]
pub enum Status {
    Closed,
    Opened,
}

impl From<Status> for ZBytes {
    fn from(value: Status) -> Self {
        <&'static str>::from(value).into()
    }
}

pub struct RateProducer<'a> {
    pub rate: Rate,
    publisher: zenoh::pubsub::Publisher<'a>,
    running_sender: Sender<bool>,
    running_receiver: Receiver<bool>,
}

impl<'a> RateProducer<'a> {
    pub async fn new(rate: Rate, key_expr: &'static str, session: &zenoh::Session) -> (Self, Sender<bool>) {
        let publisher = session
            .declare_keyexpr(key_expr)
            .await
            .and_then(|key| session.declare_publisher(key).wait())
            .unwrap();
        let (run_write, rcv) = tokio::sync::watch::channel(false);
        (RateProducer { rate, publisher, running_sender: run_write.clone(), running_receiver: rcv }, run_write)
    }
}

impl<T: Into<ZBytes>> publisher::Publisher<T> for RateProducer<'_> {
    async fn send(&self, value: T) -> zenoh::Result<()> {
        self.publisher.put(value).await
    }
}

impl publisher::IntervalProvider for RateProducer<'_> {
    fn interval(&self) -> Duration {
        self.rate.interval()
    }
}

impl publisher::Computer<Status> for RateProducer<'_> {
    fn compute(&self) -> Status {
        Status::Closed
    }
}

impl publisher::FixedIntervalPublisher<Status> for RateProducer<'_> {
    fn is_running(&self) -> bool {
        *self.running_receiver.borrow()
    }

    fn start_if_not_running(&self) -> bool {
        self.running_sender.send_if_modified(|curr| { if *curr { false } else { *curr = true; true } })
    }
}

#[tokio::main]
async fn main() {
    let session = zenoh::open(zenoh::Config::default()).await.unwrap();
    let (publisher, run_status_update) = RateProducer::new(
        Rate {
            events: 5,
            time_unit: Seconds,
        },
        "windows/studio",
        &session,
    ).await;
    let sleeper = task::spawn(async move {
        println!("sleeping 30s");
        tokio::time::sleep(Duration::from_secs(30)).await;
        println!("stopping publisher");
        run_status_update.send_replace(false);
    });
    assert!(publisher.run().await.unwrap(), "It didn't start");
    session.close().await.unwrap();
    sleeper.await.expect("This shouldn't fail");
}
