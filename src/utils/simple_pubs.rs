use crate::utils::rate::{IntervalProvider, Rate};
use crate::utils::signals::WindowStatus;
use crate::utils::signals::{NewRand, PerimeterStatus};
use std::error::Error;
use std::time::Duration;
use tokio::sync::watch::error::RecvError;
use tokio::sync::watch::{Receiver, Ref};
use tokio::task::JoinSet;
use tokio::time;
use zenoh::bytes::ZBytes;
use zenoh::pubsub::Publisher;
use zenoh::Wait;

async fn wait_until_start(running_receiver: &mut Receiver<bool>) -> Result<Ref<bool>, RecvError> {
    running_receiver.wait_for(|val| *val).await
}

// simple, but to isolate borrow
fn is_running(running_receiver: &Receiver<bool>) -> bool {
    *running_receiver.borrow()
}

async fn publish_fresh<'a, T: 'a>(
    running_watcher: &mut Receiver<bool>,
    publisher: Publisher<'_>,
    interval: Duration,
    generator: fn() -> T,
) -> Result<bool, Box<dyn Error + Send + Sync>>
where
    ZBytes: From<T>,
{
    wait_until_start(running_watcher).await?;
    println!(
        "Will publish to {} every {:?}",
        publisher.key_expr(),
        interval
    );
    let start = time::Instant::now();
    while is_running(running_watcher) {
        publisher.put(generator()).await?;
        tokio::time::sleep(interval).await;
    }
    println!(
        "Stopped {} after {:?}",
        publisher.key_expr(),
        start.elapsed()
    );
    Ok(true)
}

async fn publish_static<'a, T: 'a>(
    running_watcher: &mut Receiver<bool>,
    publisher: Publisher<'_>,
    interval: Duration,
    value: T,
) -> Result<bool, Box<dyn Error + Send + Sync>>
where
    ZBytes: From<T>,
    T: Clone,
{
    wait_until_start(running_watcher).await?;
    println!(
        "Will publish to {} every {:?}",
        publisher.key_expr(),
        interval
    );
    let start = time::Instant::now();
    while is_running(running_watcher) {
        publisher.put(value.clone()).await?;
        tokio::time::sleep(interval).await;
    }
    println!(
        "Stopped {} after {:?}",
        publisher.key_expr(),
        start.elapsed()
    );
    Ok(true)
}

pub fn publish_fixed_window_status(
    rate: Rate,
    key_expr: &'static str,
    session: &zenoh::Session,
    status: WindowStatus,
    mut running_watcher: Receiver<bool>,
    set: &mut JoinSet<Result<bool, Box<dyn Error + Send + Sync>>>,
) {
    let publisher = session
        .declare_keyexpr(format!("window/{key_expr}"))
        .wait()
        .and_then(|key| session.declare_publisher(key).wait())
        .unwrap();

    let interval = rate.interval();

    set.spawn(
        async move { publish_static(&mut running_watcher, publisher, interval, status).await },
    );
}

pub fn publish_random_window_status(
    rate: Rate,
    key_expr: &'static str,
    session: &zenoh::Session,
    running_watcher: Receiver<bool>,
    set: &mut JoinSet<Result<bool, Box<dyn Error + Send + Sync>>>,
) {
    let full_key_expr = format!("window/{key_expr}");
    let generator = WindowStatus::new_rand;
    publish_freshly(
        rate,
        session,
        full_key_expr,
        generator,
        running_watcher,
        set,
    );
}

pub fn publish_fixed_perimeter_status(
    rate: Rate,
    key_expr: &'static str,
    session: &zenoh::Session,
    status: PerimeterStatus,
    mut running_watcher: Receiver<bool>,
    set: &mut JoinSet<Result<bool, Box<dyn Error + Send + Sync>>>,
) {
    let publisher = session
        .declare_keyexpr(format!("perimeter/{key_expr}"))
        .wait()
        .and_then(|key| session.declare_publisher(key).wait())
        .unwrap();

    let interval = rate.interval();

    set.spawn(
        async move { publish_static(&mut running_watcher, publisher, interval, status).await },
    );
}

pub fn publish_random_perimeter_status(
    rate: Rate,
    key_expr: &'static str,
    session: &zenoh::Session,
    running_watcher: Receiver<bool>,
    set: &mut JoinSet<Result<bool, Box<dyn Error + Send + Sync>>>,
) {
    let full_key_expr = format!("perimeter/{key_expr}");
    let generator = PerimeterStatus::new_rand;
    publish_freshly(
        rate,
        session,
        full_key_expr,
        generator,
        running_watcher,
        set,
    );
}

pub fn publish_freshly<'a, T: 'a + 'static>(
    rate: Rate,
    session: &zenoh::Session,
    full_key_expr: String,
    generator: fn() -> T,
    mut running_watcher: Receiver<bool>,
    set: &mut JoinSet<Result<bool, Box<dyn Error + Send + Sync>>>,
) where
    ZBytes: From<T>,
{
    let publisher = session
        .declare_keyexpr(full_key_expr)
        .wait()
        .and_then(|key| session.declare_publisher(key).wait())
        .unwrap();

    let interval = rate.interval();

    set.spawn(async move {
        publish_fresh::<T>(&mut running_watcher, publisher, interval, generator).await
    });
}
