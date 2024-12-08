#[derive(Clone, Copy)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl Month {
    pub fn from_ordinal(month: u32) -> Self {
        match month {
            1 => Self::January,
            2 => Self::February,
            3 => Self::March,
            4 => Self::April,
            5 => Self::May,
            6 => Self::June,
            7 => Self::July,
            8 => Self::August,
            9 => Self::September,
            10 => Self::October,
            11 => Self::November,
            12 => Self::December,
            _ => panic!("Invalid month value: {month}"),
        }
    }

    pub fn into_ordinal(self) -> u32 {
        match self {
            Self::January => 1,
            Self::February => 2,
            Self::March => 3,
            Self::April => 4,
            Self::May => 5,
            Self::June => 6,
            Self::July => 7,
            Self::August => 8,
            Self::September => 9,
            Self::October => 10,
            Self::November => 11,
            Self::December => 12,
        }
    }
}

#[allow(clippy::needless_lifetimes)]
pub(super) mod serde_month {
    const SENTINEL_NONE: u32 = 53271237;

    use std::fmt;

    use super::Month;
    use serde::{
        de::{Error as DeError, Visitor},
        Deserializer, Serializer,
    };

    pub(in super::super) fn serialize<S>(mouth: &Option<Month>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match mouth {
            Some(mouth) => mouth.into_ordinal(),
            None => SENTINEL_NONE,
        };
        ser.serialize_u32(value)
    }

    pub(in super::super) fn deserialize<'de, D>(de: D) -> Result<Option<Month>, D::Error>
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

        let month = de.deserialize_u32(U32Visitor)?;

        if month == SENTINEL_NONE {
            Ok(None)
        } else {
            Ok(Some(Month::from_ordinal(month)))
        }
    }
}
