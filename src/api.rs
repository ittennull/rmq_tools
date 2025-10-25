use crate::dtos::Queue;
use std::sync::Arc;

use anyhow::Result;
use axum::extract::State;
use axum::Json;
use axum_macros::debug_handler;
use rabbitmq_http_client::api::Client;
use tokio::sync::Mutex;

pub struct RmqClient {
    client: Client<String, String, String>,
    vhost: String,
}

impl RmqClient {
    pub fn new(client: Client<String, String, String>, vhost: String) -> Self {
        Self { client, vhost }
    }
}

#[derive(Clone)]
pub struct AppState {
    rmq_client: Arc<Mutex<RmqClient>>,
}

impl AppState {
    pub fn new(rmq_client: RmqClient) -> Self {
        Self {
            rmq_client: Arc::new(Mutex::new(rmq_client)),
        }
    }
}

#[debug_handler]
pub async fn list_queues(State(state): State<AppState>) -> Result<Json<Vec<Queue>>, String> {
    let rmq_client = state.rmq_client.lock().await;

    let queues: Vec<_> = rmq_client
        .client
        .list_queues_in(&rmq_client.vhost)
        .await
        .map_err(|x| format!("{:?}", x))?
        .into_iter()
        .map(|q| Queue {
            name: q.name,
            message_count: q.message_count,
            exclusive: q.exclusive,
        })
        .collect();

    Ok(Json::from(queues))
}

pub async fn load_messages() -> Result<()> {
    Ok(())
}

pub async fn send_messages() -> Result<()> {
    Ok(())
}

pub async fn delete_messages() -> Result<()> {
    Ok(())
}

pub async fn update_message() -> Result<()> {
    Ok(())
}

pub async fn clear_queue() -> Result<()> {
    Ok(())
}
