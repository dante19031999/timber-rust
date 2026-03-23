use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Timestamp {
    Seconds(u64),
    DecimalSeconds(f64),
    SecondsNanoseconds(u64, u32),
}

impl From<SystemTime> for Timestamp {
    fn from(t: SystemTime) -> Self {
        let duration = t
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!");
        Timestamp::from(duration)
    }
}

impl From<Timestamp> for SystemTime {
    fn from(value: Timestamp) -> Self {
        let duration = Duration::from(value);
        let time = SystemTime::UNIX_EPOCH;
        time + duration
    }
}

impl From<Timestamp> for Duration {
    fn from(value: Timestamp) -> Self {
        match value {
            Timestamp::Seconds(seconds) => Duration::new(seconds, 0),
            Timestamp::DecimalSeconds(seconds) => {
                let secs = seconds.floor() as u64;
                let nsecs = (seconds.fract() * 1e9) as u32;
                Duration::new(secs, nsecs)
            }
            Timestamp::SecondsNanoseconds(secs, nsecs) => Duration::new(secs, nsecs),
        }
    }
}

impl From<Duration> for Timestamp {
    fn from(value: Duration) -> Self {
        Timestamp::SecondsNanoseconds(value.as_secs(), value.subsec_nanos())
    }
}
