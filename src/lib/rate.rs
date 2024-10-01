use std::{panic, time};

pub enum TimeUnit {
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
}

pub fn events_per_unit(n: u64, unit: TimeUnit) -> time::Duration {
    match unit {
        TimeUnit::Nanoseconds => {
            if n != 1 {
                panic!("Cannot have duration for more than 1 event per nanosecond")
            } else {
                time::Duration::from_nanos(n)
            }
        }
        TimeUnit::Microseconds => time::Duration::from_nanos(1000u64 / n),
        TimeUnit::Milliseconds => time::Duration::from_micros(1000u64 / n),
        TimeUnit::Seconds => time::Duration::from_millis(1000u64 / n),
        TimeUnit::Minutes => time::Duration::from_secs_f64((1f64 / (n as f64)) * 60f64),
        TimeUnit::Hours => time::Duration::from_secs_f64((1f64 / (n as f64)) * 60f64 * 60f64),
        TimeUnit::Days => {
            time::Duration::from_secs_f64((1f64 / (n as f64)) * 60f64 * 60f64 * 24f64)
        }
        TimeUnit::Weeks => {
            time::Duration::from_secs_f64((1f64 / (n as f64)) * 60f64 * 60f64 * 24f64 * 7f64)
        }
    }
}
