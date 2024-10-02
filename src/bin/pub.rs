use samp::utils;
use samp::utils::signals::{PerimeterStatus, WindowStatus};
use samp::utils::simple_pubs::{publish_fixed_perimeter_status, publish_fixed_window_status, publish_random_perimeter_status, publish_random_window_status};
use tokio::task::JoinSet;
use utils::rate::Rate;
use utils::rate::TimeUnit::Seconds;

#[tokio::main]
async fn main() {
    let session = zenoh::open(zenoh::Config::default()).await.unwrap();
    let (running_write, run_watch) = tokio::sync::watch::channel(true);
    let mut set = JoinSet::new();
    publish_fixed_window_status(
        Rate {
            events: 5,
            per_unit_of: Seconds,
        },
        "studio",
        &session,
        WindowStatus::Closed,
        run_watch.clone(),
        &mut set,
    );
    publish_fixed_window_status(
        Rate {
            events: 5,
            per_unit_of: Seconds,
        },
        "mezzanine",
        &session,
        WindowStatus::Opened,
        run_watch.clone(),
        &mut set,
    );
    publish_random_window_status(
        Rate {
            events: 5,
            per_unit_of: Seconds,
        },
        "front",
        &session,
        run_watch.clone(),
        &mut set,
    );
    publish_fixed_perimeter_status(
        Rate {
            events: 5,
            per_unit_of: Seconds,
        },
        "front",
        &session,
        PerimeterStatus::SlightMovement,
        run_watch.clone(),
        &mut set,
    );
    publish_fixed_perimeter_status(
        Rate {
            events: 5,
            per_unit_of: Seconds,
        },
        "back",
        &session,
        PerimeterStatus::NoMovement,
        run_watch.clone(),
        &mut set,
    );
    publish_random_perimeter_status(
        Rate {
            events: 5,
            per_unit_of: Seconds,
        },
        "side",
        &session,
        run_watch.clone(),
        &mut set,
    );
    running_write.send_replace(true);
    let sleeper = tokio::task::spawn(async move {
        println!("sleeping 30s");
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        println!("stopping publisher");
        running_write.send_replace(false);
    });
    while let Some(res) = set.join_next().await {
        let out = res.expect("Failed to get result").expect("Publishing thread failed");
        assert!(out, "Thread didn't start");
    }
    session.close().await.unwrap();
    sleeper.await.expect("This shouldn't fail");
}
