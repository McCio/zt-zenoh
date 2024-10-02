use crate::rate::Rate;
use std::time::Duration;
use tokio::time;
use zenoh::bytes::ZBytes;

pub trait Computer<T> {
    fn compute(&self) -> T;
}

pub trait IntervalProvider {
    fn interval(&self) -> Duration;
}

impl IntervalProvider for Rate {
    fn interval(&self) -> Duration {
        self.into()
    }
}

pub trait Publisher<T: Into<ZBytes>> {
    async fn send(&self, value: T) -> zenoh::Result<()>;
}

pub trait FixedIntervalPublisher<T: Into<ZBytes>>: Computer<T> + Publisher<T> + IntervalProvider {
    fn is_running(&self) -> bool;
    fn start_if_not_running(&self) -> bool;

    async fn run(&self) -> zenoh::Result<bool> {
        if !self.start_if_not_running() {
            return Err(zenoh::Error::from("Cannot run multiple times"));
        }
        let interval = self.interval();
        println!("Will publish every {:?}", interval);
        let start = time::Instant::now();
        while self.is_running() {
            self.send(self.compute()).await.expect("Couldn't publish");
            tokio::time::sleep(interval).await
        }
        println!("Stopped after {:?}", start.elapsed());
        Ok(true)
    }
}
