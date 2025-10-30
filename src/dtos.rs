use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
pub struct RmqConnectionInfo {
    pub domain: String,
    pub vhost: String,
}

#[derive(Serialize)]
pub struct Message {
    pub id: u64,
    pub payload: String,
}

#[derive(Deserialize)]
pub struct LoadMessagesByQueueNameQuery {
    pub queue_name: String,
}

#[derive(Serialize)]
pub struct LoadMessagesByQueueNameResponse {
    pub messages: Vec<Message>,
}

#[derive(Serialize)]
pub struct QueueSummary {
    pub name: String,
    pub message_count_in_rmq: u64,
    pub message_count_in_db: Option<u64>,
    pub exclusive: bool,
}

#[derive(Deserialize)]
pub struct DeleteMessagesRequest {
    pub message_ids: Vec<u64>,
}
