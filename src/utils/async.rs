use std::thread::park_timeout;
use tokio::time::Duration;
use tokio::time::Instant;

pub fn sleep_parking_seconds(sleeping_seconds: u32) {
    dbg!(sleeping_seconds);
    sleep_parking(Duration::from_secs(sleeping_seconds as u64));
}

pub fn sleep_parking(timeout: Duration) {
    let mut timeout_remaining = timeout;
    let beginning_park = Instant::now();
    loop {
        park_timeout(timeout_remaining);
        let elapsed = beginning_park.elapsed();
        if elapsed >= timeout {
            break;
        }
        timeout_remaining = timeout - elapsed;
        dbg!(timeout_remaining);
    }
}
