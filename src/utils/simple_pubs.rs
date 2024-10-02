use crate::utils::rate::{IntervalProvider, Rate};
use crate::utils::signals::WindowStatus;
use crate::utils::signals::{NewRand, PerimeterStatus};
use std::error::Error;
use tokio::sync::watch::error::RecvError;
use tokio::sync::watch::{Receiver, Ref, Sender};
use tokio::task::JoinSet;
use tokio::time;
use zenoh::Wait;

fn start_if_not_running(running_sender: &Sender<bool>) -> bool {
    running_sender.send_if_modified(|curr| {
        if *curr {
            false
        } else {
            *curr = true;
            true
        }
    })
}

async fn wait_until_start(running_receiver: &mut Receiver<bool>) -> Result<Ref<bool>, RecvError> {
    running_receiver.wait_for(|val| *val).await
}

// simple, but to isolate borrow
fn is_running(running_receiver: &Receiver<bool>) -> bool {
    *running_receiver.borrow()
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

    set.spawn(async move {
        running_watcher.wait_for(|val| *val).await.unwrap();
        println!("Will publish to {} every {:?}", publisher.key_expr(), interval);
        let start = time::Instant::now();
        while is_running(&running_watcher) {
            publisher.put(&status).await.expect("Couldn't publish");
            tokio::time::sleep(interval).await
        }
        println!("Stopped {} after {:?}", publisher.key_expr(), start.elapsed());
        Ok(true)
    });
}

pub fn publish_random_window_status(
    rate: Rate,
    key_expr: &'static str,
    session: &zenoh::Session,
    mut running_watcher: Receiver<bool>,
    set: &mut JoinSet<Result<bool, Box<dyn Error + Send + Sync>>>,
) {
    let publisher = session
        .declare_keyexpr(format!("window/{key_expr}"))
        .wait()
        .and_then(|key| session.declare_publisher(key).wait())
        .unwrap();

    let interval = rate.interval();

    set.spawn(async move {
        running_watcher.wait_for(|val| *val).await.unwrap();
        println!("Will publish to {} every {:?}", publisher.key_expr(), interval);
        let start = time::Instant::now();
        while is_running(&running_watcher) {
            publisher.put(&WindowStatus::new_rand()).await.expect("Couldn't publish");
            tokio::time::sleep(interval).await
        }
        println!("Stopped {} after {:?}", publisher.key_expr(), start.elapsed());
        Ok(true)
    });
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

    set.spawn(async move {
        running_watcher.wait_for(|val| *val).await.unwrap();
        println!("Will publish to {} every {:?}", publisher.key_expr(), interval);
        let start = time::Instant::now();
        while is_running(&running_watcher) {
            publisher.put(&status).await.expect("Couldn't publish");
            tokio::time::sleep(interval).await
        }
        println!("Stopped {} after {:?}", publisher.key_expr(), start.elapsed());
        Ok(true)
    });
}

pub fn publish_random_perimeter_status(
    rate: Rate,
    key_expr: &'static str,
    session: &zenoh::Session,
    mut running_watcher: Receiver<bool>,
    set: &mut JoinSet<Result<bool, Box<dyn Error + Send + Sync>>>,
) {
    let publisher = session
        .declare_keyexpr(format!("perimeter/{key_expr}"))
        .wait()
        .and_then(|key| session.declare_publisher(key).wait())
        .unwrap();

    let interval = rate.interval();

    set.spawn(async move {
        running_watcher.wait_for(|val| *val).await.unwrap();
        println!("Will publish to {} every {:?}", publisher.key_expr(), interval);
        let start = time::Instant::now();
        while is_running(&running_watcher) {
            publisher.put(&PerimeterStatus::new_rand()).await.expect("Couldn't publish");
            tokio::time::sleep(interval).await
        }
        println!("Stopped {} after {:?}", publisher.key_expr(), start.elapsed());
        Ok(true)
    });
}
