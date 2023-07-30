use serde::{de, Deserialize, Deserializer, Serialize};
use std::{fmt, num::ParseIntError, str::FromStr};

#[derive(Debug, Serialize, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Default)]
pub struct SteamId(u64);

impl SteamId {
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

impl From<u64> for SteamId {
    fn from(value: u64) -> Self {
        SteamId(value)
    }
}

impl fmt::Display for SteamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for SteamId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SteamId(s.parse()?))
    }
}

impl<'de> Deserialize<'de> for SteamId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = SteamId;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("SteamId")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SteamId(v))
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                SteamId::from_str(v).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}
