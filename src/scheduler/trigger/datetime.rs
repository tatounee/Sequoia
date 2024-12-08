use std::sync::Arc;

use chrono::{Local, NaiveDateTime};
use serde_derive::{Deserialize, Serialize};
use tokio::{
    sync::{mpsc::Sender, Mutex},
    task::JoinHandle,
};
use tracing::{debug, error};

pub use chrono::NaiveTime;

mod day;
mod month;
mod partial_date;
mod serde_time;
mod serde_year;

pub use day::Day;
pub use month::Month;
pub use partial_date::PartialDate;

#[derive(Clone, Serialize, Deserialize)]
pub struct DatetimeTrigger {
    #[serde(flatten)]
    date: PartialDate,
    #[serde(with = "serde_time")]
    time: NaiveTime,
}

impl DatetimeTrigger {

    pub fn new(date: PartialDate, time: NaiveTime) -> Self {
        Self { date, time }
    }

    pub(super) fn start(&self, generation: Arc<Mutex<u64>>, tx: Sender<u64>) -> JoinHandle<()> {
        let date = self.date;
        let time = self.time;

        tokio::spawn(async move {
            let now = Local::now();

            // if time <= now.time() {
            //     time = now.time().overflowing_add_signed(TimeDelta::seconds(8)).0;
            // }

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
        })
    }
}
