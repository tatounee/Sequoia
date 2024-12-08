use std::sync::Arc;

use tokio::{
    sync::{mpsc::Sender, Mutex},
    task::JoinHandle,
};
use tracing::debug;

use super::Trigger;

pub struct CounterTrigger {
    counter: Counter,
    trigger_model: Box<Trigger>,
    trigger: Arc<Mutex<Option<Box<Trigger>>>>,
}

#[derive(Clone, Copy)]
pub enum Counter {
    Finit(u64),
    Infinite,
}

impl CounterTrigger {
    pub fn new(counter: Counter, trigger_model: Trigger) -> Self {
        Self {
            counter,
            trigger_model: Box::new(trigger_model),
            trigger: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&self, generation: Arc<Mutex<u64>>, tx: Sender<u64>) -> JoinHandle<()> {
        let counter = self.counter;
        let trigger_model = self.trigger_model.clone();
        let trigger_storage = self.trigger.clone();

        tokio::spawn(async move {
            match counter {
                Counter::Finit(max) => {
                    for _ in 0..max {
                        let mut trigger = trigger_model.clone();
                        trigger.forward_generation(*generation.lock().await).await;
                        let mut rx = trigger.receiver().unwrap();
                        trigger.start();
                        *trigger_storage.lock().await = Some(trigger);

                        rx.recv().await;
                        let mut generation = generation.lock().await;
                        debug!("Send {generation}");

                        tx.send(*generation).await.unwrap();
                        *generation += 1;
                    }
                }
                Counter::Infinite => loop {
                    let mut trigger = trigger_model.clone();
                    trigger.forward_generation(*generation.lock().await).await;
                    let mut rx = trigger.receiver().unwrap();
                    trigger.start();
                    *trigger_storage.lock().await = Some(trigger);

                    rx.recv().await;
                    let mut generation = generation.lock().await;
                    debug!("Send {generation}");

                    tx.send(*generation).await.unwrap();
                    *generation += 1;
                },
            }
        })
    }
}

impl Clone for CounterTrigger {
    fn clone(&self) -> Self {
        Self {
            counter: self.counter,
            trigger_model: self.trigger_model.clone(),
            trigger: Arc::new(Mutex::new(None)),
        }
    }
}
