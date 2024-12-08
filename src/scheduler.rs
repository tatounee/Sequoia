use std::{future::Future, sync::Arc};

use color_eyre::eyre::ContextCompat;
use tokio::task::JoinHandle;
use tracing::debug;
use trigger::Trigger;

use crate::mailer::Mailer;

pub mod trigger;

pub struct Scheduler {
    mailer: Arc<Mailer<'static>>,
    tasks: Vec<Trigger>,
    actions: Vec<JoinHandle<()>>,
}

impl Scheduler {
    pub fn new(mailer: Mailer<'static>) -> Self {
        Self {
            mailer: Arc::new(mailer),
            tasks: Vec::new(),
            actions: Vec::new(),
        }
    }

    pub fn register_trigger_with_action<A, Fut>(
        &mut self,
        mut trigger: Trigger,
        // mut action: impl Fn(u64) -> BoxFuture<'static, ()> + Send + 'static,
        action: A,
    ) where
        A: Fn(u64, Arc<Mailer<'static>>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
        // where Fut: Fn(u64) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + 'static,
    {
        let mut rx = trigger
            .receiver()
            .context("Taking receiver from trigger in Scheduler::register_trigger_with_action")
            .unwrap();

        let mailer = self.mailer.clone();
        let action = tokio::spawn(async move {
            // Move Mailer inside the async task
            let mailer = mailer;

            while let Some(generation) = rx.recv().await {
                action(generation, mailer.clone()).await
            }

            debug!("End of action");
        });

        debug!("Start trigger");
        trigger.start();

        self.tasks.push(trigger);
        self.actions.push(action);
    }
}
