use serde::{Serialize, Deserialize};
use crate::types::UniqueId;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct GameServerId(UniqueId);

impl GameServerId {
    pub fn new() -> Self {
        Self(UniqueId::new())
    }

    pub fn try_parse(input: &str) -> Option<Self> {
        UniqueId::try_parse(input).map(Self)
    }
}

impl Default for GameServerId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<UniqueId> for GameServerId {
    fn from(value: UniqueId) -> Self {
        Self(value)
    }
}

impl fmt::Display for GameServerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "redis")]
impl rustis::resp::ToArgs for GameServerId {
    fn write_args(&self, args: &mut rustis::resp::CommandArgs) {
        args.arg(self.0);
    }
}

#[cfg(feature = "redis")]
impl rustis::resp::SingleArg for GameServerId {}

#[cfg(feature = "redis")]
impl rustis::resp::PrimitiveResponse for GameServerId {}