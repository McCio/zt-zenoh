use std::panic;
use tokio::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum TimeUnit {
    Nanoseconds,
    Microseconds,
    Milliseconds,
    #[default]
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Rate {
    pub events: u64,
    pub time_unit: TimeUnit,
}

fn events_per_unit(n: u64, unit: &TimeUnit) -> Duration {
    match unit {
        TimeUnit::Nanoseconds => {
            if n != 1 {
                panic!("Cannot have duration for more than 1 event per nanosecond")
            } else {
                Duration::from_nanos(n)
            }
        }
        TimeUnit::Microseconds => Duration::from_nanos(1000u64 / n),
        TimeUnit::Milliseconds => Duration::from_micros(1000u64 / n),
        TimeUnit::Seconds => Duration::from_millis(1000u64 / n),
        TimeUnit::Minutes => Duration::from_secs_f64((1f64 / (n as f64)) * 60f64),
        TimeUnit::Hours => Duration::from_secs_f64((1f64 / (n as f64)) * (60 * 60) as f64),
        TimeUnit::Days => Duration::from_secs_f64((1f64 / (n as f64)) * (60 * 60 * 24) as f64),
        TimeUnit::Weeks => Duration::from_secs_f64((1f64 / (n as f64)) * (60 * 60 * 24 * 7) as f64),
    }
}

impl Into<Duration> for &Rate {
    fn into(self) -> Duration {
        events_per_unit(self.events, &self.time_unit)
    }
}
