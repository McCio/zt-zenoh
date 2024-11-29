use samp::utils;
use samp::utils::base_pubs::*;
use samp::utils::r#async::{sleep_parking, sleep_parking_seconds};
use samp::utils::rate::IntervalProvider;
use std::io;
use tokio::task::JoinSet;
use utils::rate::Rate;
use utils::rate::TimeUnit::Seconds;

#[tokio::main]
async fn main() {
    let session = zenoh::open(zenoh::Config::default()).await.unwrap();
    let (running_write, run_watch) = tokio::sync::watch::channel(true);
    let mut set: JoinSet<Result<bool, ()>> = JoinSet::new();
    let rate = Rate {
        events: 2,
        per_unit_of: Seconds,
    };
    let interval = rate.interval();

    let what = String::from("window");
    let place = String::from("front");
    let mut watcher = run_watch.clone();
    let wpsession = session.clone();
    set.spawn(async move {
        let selector = format!("runtime_count/{}/{}", what, place);
        wait_until_start(&mut watcher).await.unwrap();
        while is_running(&watcher) {
            let receiver = wpsession.get(&selector).await.unwrap();
            let resp = receiver.recv_async().await.unwrap();
            let count = u64::from_be_bytes(
                resp.result()
                    .map_err(|_| io::ErrorKind::NotSeekable)
                    .and_then(|r| {
                        let bs = r.payload().to_bytes();
                        if bs.len() == 8 {
                            let carr: [u8; 8] = bs[0..8].try_into().unwrap();
                            Ok(carr)
                        } else {
                            Err(io::ErrorKind::InvalidData)
                        }
                    })
                    .unwrap(),
            );
            println!("{} {} count: {}", what, place, count);
            sleep_parking(interval);
        }
        Ok(true)
    });

    let what = String::from("window");
    let place = String::from("studio");
    let mut watcher = run_watch.clone();
    let wpsession = session.clone();
    set.spawn(async move {
        let selector = format!("runtime_count/{}/{}", what, place);
        wait_until_start(&mut watcher).await.unwrap();
        while is_running(&watcher) {
            let receiver = wpsession.get(&selector).await.unwrap();
            let resp = receiver.recv_async().await.unwrap();
            let count = u64::from_be_bytes(
                resp.result()
                    .map_err(|_| io::ErrorKind::NotSeekable)
                    .and_then(|r| {
                        let bs = r.payload().to_bytes();
                        if bs.len() == 8 {
                            let carr: [u8; 8] = bs[0..8].try_into().unwrap();
                            Ok(carr)
                        } else {
                            Err(io::ErrorKind::InvalidData)
                        }
                    })
                    .unwrap(),
            );
            println!("{} {} count: {}", what, place, count);
            sleep_parking(interval);
        }
        Ok(true)
    });

    running_write.send_replace(true);
    set.spawn(async move {
        sleep_parking_seconds(30);
        println!("stopping managed");
        running_write.send_replace(false);
        Ok(true)
    });
    sleep_parking_seconds(30);
    while let Some(res) = set.join_next().await {
        let out = res
            .expect("Failed to get result") // join error
            .expect("Publishing thread failed"); // inner-thread error
        assert!(out, "Thread didn't start");
    }
}
