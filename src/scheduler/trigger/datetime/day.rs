use std::num::NonZero;

use chrono::Weekday;

#[derive(Clone, Copy)]
pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
    Ordinal(NonZero<u32>),
}

impl Day {
    pub(super) fn try_into_chrono_weekday(self) -> Result<Weekday, ()> {
        match self {
            Self::Monday => Ok(Weekday::Mon),
            Self::Tuesday => Ok(Weekday::Tue),
            Self::Wednesday => Ok(Weekday::Wed),
            Self::Thursday => Ok(Weekday::Thu),
            Self::Friday => Ok(Weekday::Fri),
            Self::Saturday => Ok(Weekday::Sat),
            Self::Sunday => Ok(Weekday::Sun),
            _ => Err(()),
        }
    }
}

#[allow(clippy::needless_lifetimes)]
pub(super) mod serde_day {
    const SENTINEL_NONE: u32 = 53271237;
    const DB_DAY_ORDINAL_OFFSET: u32 = 7;

    use std::{fmt, num::NonZero};

    use super::Day;
    use serde::{
        de::{Error as DeError, Visitor},
        Deserializer, Serializer,
    };

    pub(in super::super) fn serialize<S>(day: &Option<Day>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match day {
            Some(day) => as_db_value(*day),
            None => SENTINEL_NONE,
        };
        ser.serialize_u32(value)
    }

    pub(in super::super) fn deserialize<'de, D>(de: D) -> Result<Option<Day>, D::Error>
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

        let day = de.deserialize_u32(U32Visitor)?;

        if day == SENTINEL_NONE {
            Ok(None)
        } else {
            Ok(Some(from_db_value(day)))
        }
    }

    fn as_db_value(day: Day) -> u32 {
        match day {
            Day::Monday => 1,
            Day::Tuesday => 2,
            Day::Wednesday => 3,
            Day::Thursday => 4,
            Day::Friday => 5,
            Day::Saturday => 6,
            Day::Sunday => 7,
            Day::Ordinal(x) => DB_DAY_ORDINAL_OFFSET + x.get(),
        }
    }

    fn from_db_value(value: u32) -> Day {
        match value {
            1 => Day::Monday,
            2 => Day::Tuesday,
            3 => Day::Wednesday,
            4 => Day::Thursday,
            5 => Day::Friday,
            6 => Day::Saturday,
            7 => Day::Sunday,
            x => Day::Ordinal(NonZero::new(x - DB_DAY_ORDINAL_OFFSET).unwrap()),
        }
    }
}
