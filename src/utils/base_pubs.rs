use crate::utils::rate::{IntervalProvider, Rate};
use std::error::Error;
use tokio::sync::watch::error::RecvError;
use tokio::sync::watch::{Receiver, Ref};
use tokio::task::JoinSet;
use tokio::time;
use zenoh::bytes::ZBytes;
use zenoh::key_expr::KeyExpr;
use zenoh::Wait;

pub async fn wait_until_start(running_receiver: &mut Receiver<bool>) -> Result<Ref<bool>, RecvError> {
    running_receiver.wait_for(|val| *val).await
}

// simple, but to isolate borrow
pub fn is_running(running_receiver: &Receiver<bool>) -> bool {
    *running_receiver.borrow()
}

pub fn publish_rate<'a, F, T, TryIntoKeyExpr>(
    set: &mut JoinSet<Result<bool, Box<dyn Error + Send + Sync>>>,
    mut running_watcher: Receiver<bool>,
    session: zenoh::Session,
    full_key_expr: TryIntoKeyExpr,
    rate: Rate<>,
    generator: F,
) where
    F: (Fn() -> T) + Send + 'a + 'static,
    T: 'a + 'static,
    ZBytes: From<T>,
    TryIntoKeyExpr: TryInto<KeyExpr<'a>> + Send + 'static,
    <TryIntoKeyExpr as TryInto<KeyExpr<'a>>>::Error: Into<zenoh::Error>,
{
    let interval = rate.interval();

    set.spawn(async move {
        let publisher = session.declare_publisher(full_key_expr).wait().unwrap();
        wait_until_start(&mut running_watcher).await?;
        println!(
            "Will publish to {} every {:?}",
            publisher.key_expr(),
            interval
        );
        let start = time::Instant::now();
        while is_running(&running_watcher) {
            publisher.put(generator()).await?;
            tokio::time::sleep(interval).await;
        }
        println!(
            "Stopped {} after {:?}",
            publisher.key_expr(),
            start.elapsed()
        );
        Ok(true)
    });
}
