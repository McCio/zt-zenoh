use samp::utils;
use samp::utils::signals::{NewRand, PerimeterStatus, RandFloat, RandInt, RandUint, WindowStatus};
use samp::utils::base_pubs::*;
use tokio::task::JoinSet;
use samp::utils::r#async::sleep_parking_seconds;
use utils::rate::Rate;
use utils::rate::TimeUnit::Seconds;

#[tokio::main]
async fn main() {
    let session = zenoh::open(zenoh::Config::default()).await.unwrap();
    let (running_write, run_watch) = tokio::sync::watch::channel(true);
    let mut set = JoinSet::new();
    publish_rate(
        &mut set,
        run_watch.clone(),
        session.clone(),
        "window/studio",
        Rate {
            events: 5,
            per_unit_of: Seconds,
        },
        || WindowStatus::Closed,
    );
    publish_rate(
        &mut set,
        run_watch.clone(),
        session.clone(),
        "window/mezzanine",
        Rate {
            events: 5,
            per_unit_of: Seconds,
        },
        || WindowStatus::Opened,
    );
    publish_rate(
        &mut set,
        run_watch.clone(),
        session.clone(),
        "window/front",
        Rate {
            events: 5,
            per_unit_of: Seconds,
        },
        WindowStatus::new_rand,
    );
    
    publish_rate(
        &mut set,
        run_watch.clone(),
        session.clone(),
        "perimeter/front",
        Rate {
            events: 5,
            per_unit_of: Seconds,
        },
        || PerimeterStatus::SlightMovement,
    );
    publish_rate(
        &mut set,
        run_watch.clone(),
        session.clone(),
        "perimeter/back",
        Rate {
            events: 5,
            per_unit_of: Seconds,
        },
        || PerimeterStatus::NoMovement,
    );
    publish_rate(
        &mut set,
        run_watch.clone(),
        session.clone(),
        "perimeter/side",
        Rate {
            events: 5,
            per_unit_of: Seconds,
        },
        PerimeterStatus::new_rand,
    );

    publish_rate(
        &mut set,
        run_watch.clone(),
        session.clone(),
        "random/float",
        Rate {
            events: 2,
            per_unit_of: Seconds,
        },
        RandFloat::new_rand,
    );
    publish_rate(
        &mut set,
        run_watch.clone(),
        session.clone(),
        "random/int",
        Rate {
            events: 2,
            per_unit_of: Seconds,
        },
        RandInt::new_rand,
    );
    publish_rate(
        &mut set,
        run_watch.clone(),
        session.clone(),
        "random/uint",
        Rate {
            events: 2,
            per_unit_of: Seconds,
        },
        RandUint::new_rand,
    );
    running_write.send_replace(true);
    set.spawn(async move {
        sleep_parking_seconds(30);
        println!("stopping publishers");
        running_write.send_replace(false);
        Ok(true)
    });
    while let Some(res) = set.join_next().await {
        let out = res
            .expect("Failed to get result") // join error
            .expect("Publishing thread failed"); // inner-thread error
        assert!(out, "Thread didn't start");
    }
    session.close().await.unwrap();
}
