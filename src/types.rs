pub mod rmq_types {
    pub struct RemoteQueue {
        pub name: String,
        pub message_count: u64,
        pub exclusive: bool,
    }
}

pub mod db_types{
    pub struct LocalQueue {
        pub name: String,
        pub message_count: u64,
    }
}