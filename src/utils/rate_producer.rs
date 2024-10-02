use crate::utils;
use std::time::Duration;
use tokio::sync::watch::{Receiver, Sender};
use utils::publisher;
use utils::rate::Rate;
use zenoh::bytes::ZBytes;
use zenoh::Wait;

pub struct RateProducer<'a> {
    pub rate: Rate,
    publisher: zenoh::pubsub::Publisher<'a>,
    running_sender: Sender<bool>,
    running_receiver: Receiver<bool>,
}

impl<'a> RateProducer<'a> {
    pub async fn new_first(
        rate: Rate,
        key_expr: &'static str,
        session: &zenoh::Session,
    ) -> (Self, Sender<bool>) {
        let publisher = session
            .declare_keyexpr(key_expr)
            .await
            .and_then(|key| session.declare_publisher(key).wait())
            .unwrap();
        let (run_write, rcv) = tokio::sync::watch::channel(false);
        (
            RateProducer {
                rate,
                publisher,
                running_sender: run_write.clone(),
                running_receiver: rcv,
            },
            run_write,
        )
    }

    pub async fn new(
        rate: Rate,
        key_expr: &'static str,
        session: &zenoh::Session,
        sender: Sender<bool>,
    ) -> (Self, Sender<bool>) {
        let publisher = session
            .declare_keyexpr(key_expr)
            .await
            .and_then(|key| session.declare_publisher(key).wait())
            .unwrap();
        (
            RateProducer {
                rate,
                publisher,
                running_sender: sender.clone(),
                running_receiver: sender.subscribe(),
            },
            sender,
        )
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

impl publisher::RunningWatch for RateProducer<'_> {
    fn running_receiver(&self) -> &Receiver<bool> {
        &self.running_receiver
    }

    fn running_sender(&self) -> &Sender<bool> {
        &self.running_sender
    }
}
