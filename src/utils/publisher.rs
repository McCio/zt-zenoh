use crate::utils::rate::Rate;
use std::time::Duration;
use tokio::sync::watch::{Receiver, Sender};
use tokio::time;
use zenoh::bytes::ZBytes;

pub trait FutRes<VAL>: std::future::Future<Output = zenoh::Result<VAL>> {}
impl<VAL, TYP: std::future::Future<Output = zenoh::Result<VAL>>> FutRes<VAL> for TYP {}

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
    fn send(&self, value: T) -> impl FutRes<()>;
}

pub trait RunningWatch {
    fn running_receiver(&self) -> &Receiver<bool>;
    fn running_sender(&self) -> &Sender<bool>;
}

pub trait FixedIntervalPublisher<T: Into<ZBytes>>:
    Computer<T> + Publisher<T> + IntervalProvider + RunningWatch
{
    fn is_running(&self) -> bool {
        *self.running_receiver().borrow()
    }

    fn start_if_not_running(&self) -> bool {
        self.running_sender().send_if_modified(|curr| {
            if *curr {
                false
            } else {
                *curr = true;
                true
            }
        })
    }

    fn run(&self) -> impl FutRes<bool> {
        async {
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
}
