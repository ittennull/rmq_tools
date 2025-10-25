use serde::Serialize;

#[derive(Serialize)]
pub struct Queue{
    pub name: String,
    pub message_count: u64,
    pub exclusive: bool,
}