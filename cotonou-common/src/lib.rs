mod authentication;
mod dal;
pub mod http;
pub mod matchmaking;
pub mod models;
pub mod mongo;
pub mod notifications;
mod profile;
pub mod redis;
pub mod steam;

pub use authentication::*;
pub use dal::*;
pub use profile::*;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{de, Deserialize, Deserializer};

pub fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn deserialize_iso_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamp_str = String::deserialize(deserializer)?;
    Ok(DateTime::parse_from_rfc3339(&timestamp_str)
        .map_err(serde::de::Error::custom)?
        .into())
}

pub fn deserialize_unix_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let unix_timestamp = i64::deserialize(deserializer)?;
    let naive = NaiveDateTime::from_timestamp_opt(unix_timestamp, 0)
        .ok_or_else(|| de::Error::custom("Cannot parse UNIX timestamp"))?;
    Ok(DateTime::from_utc(naive, Utc))
}
