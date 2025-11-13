use crate::dtos::QueueCounters;
use crate::rabbitmq::Rabbitmq;
use log::debug;
use std::sync::{Arc};
use std::time::Duration;
use tokio::sync::watch::{Receiver, Sender};
use tokio::sync::{watch, Notify};
use tokio::time;

#[derive(Clone)]
pub struct RmqBackground {
    sender: Sender<Vec<QueueCounters>>,
    notify_worker: Arc<Notify>,
}

impl RmqBackground {
    pub fn new(rmq: Rabbitmq) -> RmqBackground {
        let notify_worker = Arc::new(Notify::new());
        let (sender, _) = watch::channel(vec![]);

        // start a task that waits for a Notify and then queries RMQ and sends counters until
        // there are receivers listening to it. After that it starts again from waiting for a Notify
        {
            let notify_worker = Arc::clone(&notify_worker);
            let sender = sender.clone();
            tokio::spawn(async move {
                loop {
                    notify_worker.notified().await;
                    update_counter(&sender, &rmq).await;
                }
            });
        }

        Self {
            sender,
            notify_worker,
        }
    }

    pub fn subscribe(&self) -> Receiver<Vec<QueueCounters>> {
        let receiver = self.sender.subscribe();
        self.notify_worker.notify_one();
        receiver
    }
}

async fn update_counter(sender: &Sender<Vec<QueueCounters>>, rmq: &Rabbitmq) {
    while let Ok(queues) = rmq.list_queues().await {
        let counters = queues
            .into_iter()
            .map(|q| QueueCounters {
                queue_name: q.name,
                messages: q.message_count,
            })
            .collect();

        if sender.send(counters).is_err() {
            debug!("No receivers left. Exiting update_counter");
            return;
        };

        time::sleep(Duration::from_secs(5)).await;
    }
}
