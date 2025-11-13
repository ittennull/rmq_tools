pub mod rmq_types {
    use rabbitmq_http_client::api::Client;

    pub type RmqClient = Client<String, String, String>;
    
    pub struct RemoteQueue {
        pub name: String,
        pub message_count: u64,
        pub exclusive: bool,
    }
}

pub mod db_types{
    use crate::database::QueueId;

    pub struct LocalQueue {
        pub id: QueueId,
        pub name: String,
        pub message_count: u64,
    }
}