use serde::Serialize;

#[derive(Serialize)]
pub struct RemoteQueue {
    pub name: String,
    pub message_count: u64,
    pub exclusive: bool,
}

#[derive(Serialize)]
pub struct LocalQueue {
    pub id: u64,
    pub name: String,
}

#[derive(Serialize)]
pub struct Message {
    pub id: u64,
    pub payload: String,
}

pub mod list_queues {
    use crate::dtos::RemoteQueue;
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct Queue {
        pub remote_queue: RemoteQueue,
        pub exists_locally: bool,
    }
}

pub mod find_queue_by_name {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub struct FindQuery {
        pub name: String,
    }

    #[derive(Serialize)]
    pub struct Response {
        pub queue_id: u64,
    }
}

pub mod delete_messages{
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Request {
        pub message_ids: Vec<u64>,
    }
}