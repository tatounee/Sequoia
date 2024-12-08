
mod datetime;
mod counter;

pub use datetime::{DatetimeTrigger, PartialDate, NaiveTime, Day, Month};
pub use counter::{CounterTrigger, Counter};

use tokio::sync::mpsc::Receiver;

pub type Action = Box<dyn FnMut()>;

pub trait Trigger {
    fn abort(&self);
    fn receiver(&mut self) -> Option<Receiver<u64>>;
    fn generation(&self) -> u64;
    fn forward_generation(&mut self, offset: u64);
}
