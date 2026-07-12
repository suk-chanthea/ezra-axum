//! `LocalTime` mirrors the Go `dto.LocalTime` type: timestamps are serialized as a
//! human-readable string in the Asia/Phnom_Penh timezone (e.g. "2025-12-05 6:50pm").

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Asia::Phnom_Penh;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Format equivalent to Go's "2006-01-02 3:04pm".
const TIME_FORMAT: &str = "%Y-%m-%d %-I:%M%P";

#[derive(Debug, Clone, Copy)]
pub struct LocalTime(pub DateTime<Utc>);

impl LocalTime {
    pub fn new(t: DateTime<Utc>) -> Self {
        LocalTime(t)
    }

    pub fn opt(t: Option<DateTime<Utc>>) -> Option<Self> {
        t.map(LocalTime)
    }
}

impl Serialize for LocalTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let local = self.0.with_timezone(&Phnom_Penh);
        serializer.serialize_str(&local.format(TIME_FORMAT).to_string())
    }
}

impl<'de> Deserialize<'de> for LocalTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // Try the local display format first.
        if let Ok(naive) = NaiveDateTime::parse_from_str(&s, TIME_FORMAT) {
            let dt = Phnom_Penh
                .from_local_datetime(&naive)
                .single()
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|| Utc.from_utc_datetime(&naive));
            return Ok(LocalTime(dt));
        }

        // Fall back to RFC3339.
        match DateTime::parse_from_rfc3339(&s) {
            Ok(dt) => Ok(LocalTime(dt.with_timezone(&Utc))),
            Err(e) => Err(serde::de::Error::custom(e)),
        }
    }
}
