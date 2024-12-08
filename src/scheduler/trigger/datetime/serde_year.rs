const SENTINEL_NONE: u32 = 53271237;

use std::fmt;

use serde::{
    de::{Error as DeError, Visitor},
    Deserializer, Serializer,
};

pub(super) fn serialize<S>(year: &Option<u32>, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value = match year {
        Some(year) => *year,
        None => SENTINEL_NONE,
    };
    ser.serialize_u32(value)
}

#[allow(clippy::needless_lifetimes)]
pub(super) fn deserialize<'de, D>(de: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    struct U32Visitor;

    impl<'de> Visitor<'de> for U32Visitor {
        type Value = u32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("")
        }

        fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            Ok(value)
        }
    }

    let year = de.deserialize_u32(U32Visitor)?;

    if year == SENTINEL_NONE {
        Ok(None)
    } else {
        Ok(Some(year))
    }
}
