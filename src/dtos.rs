use crate::database::{MessageId, QueueId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
pub struct RmqConnectionInfo {
    pub domain: String,
    pub server_name: Option<String>,
    pub vhost: String,
}

#[derive(Serialize, Clone)]
pub struct EnvInfo {
    pub rmq_connection_info: RmqConnectionInfo,
    pub importance_level: u8,
}

#[derive(Serialize)]
pub struct Message {
    pub id: MessageId,
    pub payload: String,
    pub headers: serde_json::Map<String, serde_json::Value>,
}

#[derive(Deserialize)]
pub struct LoadMessagesByQueueNameQuery {
    pub queue_name: String,
}

#[derive(Deserialize)]
pub struct PeekMessagesQuery {
    pub queue_name: String,
}

#[derive(Serialize)]
pub struct LoadMessagesByQueueNameResponse {
    pub queue_id: QueueId,
    pub messages: Vec<Message>,
}

#[derive(Serialize)]
pub struct QueueSummary {
    pub queue_id: Option<QueueId>,
    pub name: String,
    pub message_count_in_rmq: u32,
    pub message_count_in_db: u32,
    pub exclusive: bool,
}

#[derive(Deserialize)]
pub struct DeleteMessagesRequest {
    pub message_ids: Vec<MessageId>,
}

#[derive(Deserialize)]
pub struct SendMessagesRequest {
    pub message_ids: Vec<MessageId>,
    pub destination_queue_name: String,
}

#[derive(Serialize)]
pub struct QueueCounters {
    pub queue_name: String,
    pub messages: u64,
}
