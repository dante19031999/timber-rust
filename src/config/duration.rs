use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum FlexibleDuration {
    Seconds(u64),
    DecimalSeconds(f64),
    SecondsNanoseconds(u64, u32),
}

impl From<FlexibleDuration> for Duration {
    fn from(value: FlexibleDuration) -> Self {
        match value {
            FlexibleDuration::Seconds(seconds) => Duration::new(seconds, 0),
            FlexibleDuration::DecimalSeconds(seconds) => {
                let secs = seconds.floor() as u64;
                let nsecs = (seconds.fract() * 1e9) as u32;
                Duration::new(secs, nsecs)
            }
            FlexibleDuration::SecondsNanoseconds(secs, nsecs) => Duration::new(secs, nsecs),
        }
    }
}

impl From<Duration> for FlexibleDuration {
    fn from(value: Duration) -> Self {
        FlexibleDuration::SecondsNanoseconds(value.as_secs(), value.subsec_nanos())
    }
}
