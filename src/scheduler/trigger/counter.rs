use std::sync::Arc;

use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    task::JoinHandle,
};
use tracing::debug;

use super::Trigger;

pub struct CounterTrigger {
    counter: Counter,
    generation: Arc<Mutex<u64>>,
    handler: JoinHandle<()>,
    sender: Sender<u64>,
    receiver: Option<Receiver<u64>>,
    trigger_factory: Arc<dyn Fn() -> Box<dyn Trigger + Send>>,
    trigger: Arc<Mutex<Option<Box<dyn Trigger + Send>>>>,
}

#[derive(Clone, Copy)]
pub enum Counter {
    Finit(u64),
    Infinite,
}

impl CounterTrigger {
    pub fn new(
        counter: Counter,
        trigger_factory: impl Fn() -> Box<dyn Trigger + Send> + Send + Sync + 'static,
    ) -> Self {
        let (tx, rx) = mpsc::channel(4);
        let tx_ = tx.clone();

        let generation = Arc::new(Mutex::new(0));
        let generation_ = generation.clone();

        let trigger_factory = Arc::new(trigger_factory);
        let trigger_factory_ = trigger_factory.clone();

        let trigger = Arc::new(Mutex::new(None));
        let trigger_ = trigger.clone();

        let handler = tokio::spawn(async move {
            let generation = generation_;
            let trigger_factory = trigger_factory_;
            let trigger_ = trigger_;

            match counter {
                Counter::Finit(max) => {
                    for _ in 0..max {
                        let mut trigger = trigger_factory();
                        let mut rx = trigger.receiver().unwrap();
                        *trigger_.lock().await = Some(trigger);

                        rx.recv().await;
                        let mut generation = generation.lock().await;
                        debug!("Send {generation}");
            
                        tx.send(*generation).await.unwrap();
                        *generation += 1;
                    }
                }
                Counter::Infinite => loop {
                    let mut trigger = trigger_factory();
                    let mut rx = trigger.receiver().unwrap();
                    *trigger_.lock().await = Some(trigger);

                    rx.recv().await;
                    let mut generation = generation.lock().await;
                    debug!("Send {generation}");
        
                    tx.send(*generation).await.unwrap();
                    *generation += 1;
                },
            }
        });

        Self {
            counter,
            generation,
            handler,
            sender: tx_,
            receiver: Some(rx),
            trigger_factory,
            trigger,
        }
    }
}

impl Trigger for CounterTrigger {
    fn abort(&self) {
        self.handler.abort();
        tokio::task::block_in_place(|| {
            let trigger = self.trigger.blocking_lock().take();
            if let Some(trigger) = trigger {
                trigger.abort();
            }
        })
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
