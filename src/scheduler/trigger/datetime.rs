use std::sync::Arc;

use chrono::{Datelike, Days, Local, NaiveDate, NaiveDateTime, Weekday};
use color_eyre::eyre::Result;
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    task::JoinHandle,
};
use tracing::{debug, error, warn};

use super::Trigger;

pub use chrono::NaiveTime;

pub struct DatetimeTrigger {
    date: PartialDate,
    time: NaiveTime,
    handler: JoinHandle<()>,
    sender: Sender<u64>,
    receiver: Option<Receiver<u64>>,
    generation: Arc<Mutex<u64>>,
}

impl DatetimeTrigger {
    pub fn new(date: PartialDate, time: NaiveTime) -> Self {
        let (tx, rx) = mpsc::channel(4);
        let tx_ = tx.clone();

        let generation = Arc::new(Mutex::new(0));
        let generation_ = generation.clone();

        let handler = tokio::spawn(async move {
            let generation = generation_;

            let now = Local::now();
            let target = NaiveDateTime::new(date.next_valide_date(time), time)
                .and_local_timezone(Local)
                .unwrap();

            let duration = (target - now).to_std().unwrap();

            debug!("now = {now:?}");
            debug!("target = {target:?}");
            debug!("duration = {duration:?}");

            tokio::time::sleep(duration).await;

            let generation = *generation.lock().await;
            debug!("Send {generation}");

            let res = tx.send(generation).await;
            if let Err(err) = res {
                error!("{err:?}");
            }
        });

        Self {
            date,
            time,
            handler,
            sender: tx_,
            receiver: Some(rx),
            generation,
        }
    }
}

impl Trigger for DatetimeTrigger {
    fn abort(&self) {
        self.handler.abort();
    }

    fn receiver(&mut self) -> Option<Receiver<u64>> {
        self.receiver.take()
    }

    fn generation(&self) -> u64 {
        tokio::task::block_in_place(|| *self.generation.blocking_lock())
    }

    fn forward_generation(&mut self, offset: u64) {
        tokio::task::block_in_place(move || {
            let mut generation = self.generation.blocking_lock();
            *generation += offset;
        })
    }
}

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
        debug_assert!(!matches!(day, Day::Ordinal(x) if x > 31 || x == 0));
        match day {
            Day::Ordinal(x) if x > 31 => {
                error!("Try to create a PartialDate with day equal to {x}, which is greater than 31. Clamp the day to 31.");
                Day::Ordinal(31)
            }
            Day::Ordinal(x) if x == 0 => {
                error!("Try to create a PartialDate with day equal to {x}, which is less than 1. Clamp the day to 1.");
                Day::Ordinal(1)
            }
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
                    x - 1
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

#[derive(Clone, Copy)]
pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
    Ordinal(u32),
}

impl Day {
    fn try_into_chrono_weekday(self) -> Result<Weekday, ()> {
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
