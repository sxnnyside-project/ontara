use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// A point in real time, as milliseconds since the Unix epoch.
///
/// Domain operations receive a `Timestamp` instead of reading the clock, so they stay deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(i64);

impl Timestamp {
    #[must_use]
    pub const fn from_unix_millis(millis: i64) -> Self {
        Self(millis)
    }

    #[must_use]
    pub const fn as_unix_millis(self) -> i64 {
        self.0
    }

    /// Reads the system clock. A clock set before the epoch yields the epoch.
    #[must_use]
    pub fn now() -> Self {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| {
                i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
            });
        Self(millis)
    }
}
