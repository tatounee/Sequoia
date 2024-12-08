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

