mod api_error;

use crate::api::api_error::ApiError;
use crate::database::Database;
use crate::dtos::{delete_messages, find_queue_by_name, list_queues, Message};
use crate::rabbitmq::Rabbitmq;
use anyhow::{anyhow, Result};
use axum::extract::{Path, Query, State};
use axum::http::HeaderValue;
use axum::routing::{delete, get};
use axum::{Json, Router};
use axum_macros::debug_handler;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

struct GuardedData {
    rabbitmq: Rabbitmq,
    database: Database,
}

#[derive(Clone)]
pub struct AppState {
    guarded: Arc<Mutex<GuardedData>>,
}

impl AppState {
    pub fn new(rabbitmq: Rabbitmq, database: Database) -> Self {
        Self {
            guarded: Arc::new(Mutex::new(GuardedData { rabbitmq, database })),
        }
    }
}

pub fn build_api(rmq_client: Rabbitmq, database: Database) -> Router {
    let state = AppState::new(rmq_client, database);

    let cors_layer = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin("http://localhost:5009".parse::<HeaderValue>().unwrap());

    Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/queues", get(list_queues))
                .route("/queue", get(find_queue_by_name))
                .route("/queues/{queue_id}/messages", get(get_messages))
                .route("/messages", delete(delete_messages))
                .with_state(state)
                .layer(cors_layer),
        )
        .fallback_service(ServeDir::new("static"))
}

#[debug_handler]
async fn list_queues(
    State(state): State<AppState>,
) -> Result<Json<Vec<list_queues::Queue>>, ApiError> {
    let guarded = state.guarded.lock().await;

    let remote_queues = guarded.rabbitmq.list_queues().await?;
    let local_queues = guarded.database.get_queues()?;

    let queues: Vec<_> = remote_queues
        .into_iter()
        .map(|remote_queue| {
            let exists_locally = local_queues.iter().any(|q| q.name == remote_queue.name);
            list_queues::Queue {
                remote_queue,
                exists_locally,
            }
        })
        .collect();

    Ok(Json::from(queues))
}

async fn find_queue_by_name(
    State(state): State<AppState>,
    Query(find_queue_by_name::FindQuery { name }): Query<find_queue_by_name::FindQuery>,
) -> Result<Json<find_queue_by_name::Response>, ApiError> {
    let guarded = state.guarded.lock().await;
    let queue_id = guarded.database.find_queue_by_name(&name)?;
    Ok(Json(find_queue_by_name::Response { queue_id }))
}

async fn get_messages(
    State(state): State<AppState>,
    Path(queue_id): Path<u64>,
) -> Result<Json<Vec<Message>>, ApiError> {
    let guarded = state.guarded.lock().await;
    let messages = guarded.database.get_messages(queue_id, 0, 100)?;
    Ok(Json(messages))
}

pub async fn send_messages() -> Result<()> {
    Ok(())
}

pub async fn delete_messages(
    State(state): State<AppState>,
    Json(request): Json<delete_messages::Request>,
) -> Result<(), ApiError> {
    if request.message_ids.is_empty() {
        return Err(ApiError::Http(anyhow!(
            "message_ids array must contain at least one id"
        )));
    }

    let guarded = state.guarded.lock().await;
    guarded.database.delete_messages(&request.message_ids)?;
    Ok(())
}

pub async fn update_message() -> Result<()> {
    Ok(())
}

pub async fn clear_queue() -> Result<()> {
    Ok(())
}
