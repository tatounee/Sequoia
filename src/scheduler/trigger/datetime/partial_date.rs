use std::num::NonZero;

use chrono::{Datelike, Days, Local, NaiveDate, NaiveTime};
use tracing::{error, warn};

use super::{Day, Month};

#[derive(Clone, Copy)]
pub struct PartialDate {
    year: Option<u32>,
    month: Option<Month>,
    day: Option<Day>,
}

impl PartialDate {
    pub fn new_y(year: u32) -> Self {
        Self {
            year: Some(Self::verify_year(year)),
            month: None,
            day: None,
        }
    }

    pub fn new_m(month: Month) -> Self {
        Self {
            year: None,
            month: Some(month),
            day: None,
        }
    }

    pub fn new_d(day: Day) -> Self {
        Self {
            year: None,
            month: None,
            day: Some(Self::verify_day(day)),
        }
    }

    pub fn new_ym(year: u32, month: Month) -> Self {
        Self {
            year: Some(Self::verify_year(year)),
            month: Some(month),
            day: None,
        }
    }

    pub fn new_yd(year: u32, day: Day) -> Self {
        Self {
            year: Some(Self::verify_year(year)),
            month: None,
            day: Some(Self::verify_day(day)),
        }
    }

    pub fn new_md(month: Month, day: Day) -> Self {
        Self {
            year: None,
            month: Some(month),
            day: Some(Self::verify_day(day)),
        }
    }

    pub fn new_ymd(year: u32, month: Month, day: Day) -> Self {
        Self {
            year: Some(Self::verify_year(year)),
            month: Some(month),
            day: Some(Self::verify_day(day)),
        }
    }

    fn verify_year(year: u32) -> u32 {
        debug_assert!(year <= 9999);
        debug_assert!(year >= 2000);
        if year > 9999 {
            warn!("Try to create a PartialDate with year equal to {year}, which is greater than 9999.");
        }
        if year < 2000 {
            warn!(
                "Try to create a PartialDate with year equal to {year}, which is less than 2000."
            );
        }
        year
    }

    fn verify_day(day: Day) -> Day {
        debug_assert!(!matches!(day, Day::Ordinal(x) if x.get() > 31));
        match day {
            Day::Ordinal(x) if x.get() > 31 => {
                error!("Try to create a PartialDate with day equal to {x}, which is greater than 31. Clamp the day to 31.");
                Day::Ordinal(NonZero::new(31).unwrap())
            }
            // Day::Ordinal(x) if x == 0 => {
            //     error!("Try to create a PartialDate with day equal to {x}, which is less than 1. Clamp the day to 1.");
            //     Day::Ordinal(1)
            // }
            _ => day,
        }
    }

    // TODO: A bien tester
    pub fn next_valide_date(&self, time: NaiveTime) -> NaiveDate {
        let now = Local::now();

        let year = self.year.map(|y| y as i32).unwrap_or(now.year());
        let month = self.month.map(Month::into_ordinal).unwrap_or(now.month());
        let mut days_offset = self
            .day
            .map(|day| {
                let current_day = now.date_naive().weekday();

                if let Ok(day) = day.try_into_chrono_weekday() {
                    let offset = day.days_since(current_day);
                    now.day() + offset - 1
                } else if let Day::Ordinal(x) = day {
                    x.get() - 1
                } else {
                    unreachable!()
                }
            })
            .unwrap_or(now.day() - 1);

        if now.time() >= time {
            days_offset += 1;
        }

        let date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        date.checked_add_days(Days::new(days_offset as u64))
            .unwrap()
    }
}
