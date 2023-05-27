use std::{fmt, num::{TryFromIntError, ParseIntError}, str::FromStr};

use mongodb::bson::Bson;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Default)]
pub struct ProfileId(u32);

#[cfg(feature = "redis")]
impl rustis::resp::ToArgs for ProfileId {
    fn write_args(&self, args: &mut rustis::resp::CommandArgs) {
        args.arg(self.0);
    }
}

#[cfg(feature = "redis")]
impl rustis::resp::SingleArg for ProfileId {}

#[cfg(feature = "redis")]
impl rustis::resp::PrimitiveResponse for ProfileId {}

impl From<ProfileId> for Bson {
    fn from(value: ProfileId) -> Self {
        value.0.into()
    }
}

impl TryFrom<i64> for ProfileId {
    type Error = TryFromIntError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(ProfileId(value.try_into()?))
    }
}

impl fmt::Display for ProfileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for ProfileId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ProfileId(s.parse()?))
    }
}
