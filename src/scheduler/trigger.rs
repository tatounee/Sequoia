mod counter;
mod datetime;

use std::sync::Arc;

pub use counter::{Counter, CounterTrigger};
pub use datetime::{DatetimeTrigger, Day, Month, NaiveTime, PartialDate};

use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};
use tracing::warn;

pub type Action = Box<dyn FnMut()>;

pub struct Trigger {
    kind: TriggerKind,
    receiver: Option<Receiver<u64>>,
    sender: Option<Sender<u64>>,
    generation: Arc<Mutex<u64>>,
}

#[derive(Clone)]
enum TriggerKind {
    Counter(CounterTrigger),
    Datetime(DatetimeTrigger),
}

impl Trigger {
    pub fn receiver(&mut self) -> Option<Receiver<u64>> {
        self.receiver.take()
    }

    pub async fn generation(&self) -> u64 {
        *self.generation.lock().await
    }

    pub async fn forward_generation(&mut self, offset: u64) {
        *self.generation.lock().await += offset;
    }

    pub fn start(&mut self) {
        if let Some(sender) = self.sender.take() {
            match &self.kind {
                TriggerKind::Datetime(trigger) => {
                    trigger.start(self.generation.clone(), sender);
                }
                TriggerKind::Counter(trigger) => {
                    trigger.start(self.generation.clone(), sender);
                }
            }
        } else {
            warn!("Trigger already started");
        }
    }
}

impl Clone for Trigger {
    fn clone(&self) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(4);

        Self {
            kind: self.kind.clone(),
            sender: Some(tx),
            receiver: Some(rx),
            generation: Arc::new(Mutex::new(0)),
        }
    }
}

impl From<DatetimeTrigger> for Trigger {
    fn from(trigger: DatetimeTrigger) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(4);

        Self {
            kind: TriggerKind::Datetime(trigger),
            sender: Some(tx),
            receiver: Some(rx),
            generation: Arc::new(Mutex::new(0)),
        }
    }
}

impl From<CounterTrigger> for Trigger {
    fn from(trigger: CounterTrigger) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(4);

        Self {
            kind: TriggerKind::Counter(trigger),
            sender: Some(tx),
            receiver: Some(rx),
            generation: Arc::new(Mutex::new(0)),
        }
    }
}
