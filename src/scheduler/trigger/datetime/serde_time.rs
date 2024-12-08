use std::fmt;

use chrono::{NaiveTime, Timelike};
use serde::{
    de::{Error as DeError, Visitor},
    Deserializer, Serializer,
};

pub(super) fn serialize<S>(time: &NaiveTime, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_u32(time.second())
}

#[allow(clippy::needless_lifetimes)]
pub(super) fn deserialize<'de, D>(de: D) -> Result<NaiveTime, D::Error>
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

    let sec = de.deserialize_u32(U32Visitor)?;
    let h = sec / 3600;
    let m = (sec % 3600) / 60;
    let s = sec % 60;

    Ok(NaiveTime::from_hms_opt(h, m, s).unwrap())
}
