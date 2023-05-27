use serde::{de, ser};
use std::fmt;
use uuid::{fmt::Simple, Uuid};

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct UniqueId(u128);

impl fmt::Debug for UniqueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0; Simple::LENGTH];
        let str = self.format(&mut buf);
        f.write_str(str)
    }
}

impl fmt::Display for UniqueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0; Simple::LENGTH];
        let str = self.format(&mut buf);
        f.write_str(str)
    }
}

#[cfg(feature = "redis")]
impl rustis::resp::ToArgs for UniqueId {
    fn write_args(&self, args: &mut rustis::resp::CommandArgs) {
        let mut buf = [0; Simple::LENGTH];
        let str = self.format(&mut buf);
        args.arg(str);
    }
}

#[cfg(feature = "redis")]
impl rustis::resp::SingleArg for UniqueId {}

#[cfg(feature = "redis")]
impl rustis::resp::PrimitiveResponse for UniqueId {}

impl UniqueId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().as_u128())
    }

    pub fn try_parse(input: &str) -> Option<Self> {
        Uuid::try_parse(input).ok().map(|id| Self(id.as_u128()))
    }

    pub fn format<'buf>(&self, buffer: &'buf mut [u8]) -> &'buf str {
        Uuid::from_u128(self.0).as_simple().encode_lower(buffer)
    }
}

impl Default for UniqueId {
    fn default() -> Self {
        Self::new()
    }
}

impl<'de> de::Deserialize<'de> for UniqueId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = UniqueId;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("u128")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(UniqueId(
                    Uuid::parse_str(v)
                        .map_err(|_| {
                            de::Error::invalid_value(de::Unexpected::Str(v), &"a valid UUID")
                        })?
                        .as_u128(),
                ))
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl ser::Serialize for UniqueId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut buf = [0; Simple::LENGTH];
        let str = self.format(&mut buf);
        serializer.serialize_str(str)
    }
}

#[cfg(test)]
mod tests {
    use super::UniqueId;

    const TEST_UUID: &str = "1f6cf4f5d977453394c6ba33b7a3e299";

    #[test]
    fn debug() {
        let unique_id = UniqueId::try_parse(TEST_UUID).unwrap();
        let str = format!("{unique_id:?}");
        assert_eq!(TEST_UUID, str);
    }

    #[test]
    fn display() {
        let unique_id = UniqueId::try_parse(TEST_UUID).unwrap();
        let str = unique_id.to_string();
        assert_eq!(TEST_UUID, str);
    }

    #[test]
    fn deserialize() {
        let expected_id = UniqueId::try_parse(TEST_UUID).unwrap();
        let actual_id = serde_json::from_str::<UniqueId>(&format!("\"{TEST_UUID}\"")).unwrap();
        assert_eq!(expected_id, actual_id);

        let result = serde_json::from_str::<UniqueId>("\"abc\"");
        println!("{result:?}");
        assert!(result.is_err());
    }

    #[test]
    fn serialize() {
        let expected_id = format!("\"{TEST_UUID}\"");
        let actual_id = serde_json::to_string(&UniqueId::try_parse(TEST_UUID)).unwrap();
        assert_eq!(expected_id, actual_id);
    }
}
