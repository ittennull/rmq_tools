mod api_error;

use crate::api::api_error::ApiError;
use crate::database::Database;
use crate::dtos::{
    DeleteMessagesRequest, LoadMessagesByQueueNameQuery, LoadMessagesByQueueNameResponse, Message,
    QueueSummary, RmqConnectionInfo,
};
use crate::rabbitmq::Rabbitmq;
use anyhow::{anyhow, Result};
use axum::extract::{Path, Query, State};
use axum::http::HeaderValue;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use axum_macros::debug_handler;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

struct GuardedData {
    rabbitmq: Rabbitmq,
    database: Database,
}

#[derive(Clone)]
pub struct AppState {
    guarded: Arc<Mutex<GuardedData>>,
    rmq_connection_info: RmqConnectionInfo,
}

impl AppState {
    pub fn new(rabbitmq: Rabbitmq, database: Database) -> Self {
        Self {
            rmq_connection_info: rabbitmq.get_connection_info(),
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
                .route("/rmq_connection", get(get_rmq_connection_info))
                .route("/queue/load", post(load_messages_by_queue_name))
                .route("/queues", get(list_queues))
                .route("/queues/{queue_id}/messages", get(get_messages))
                .route("/messages", delete(delete_messages))
                .with_state(state)
                .layer(cors_layer),
        )
        .fallback_service(ServeDir::new("static").fallback(ServeFile::new("static/index.html")))
}

async fn get_rmq_connection_info(State(state): State<AppState>) -> Json<RmqConnectionInfo> {
    Json(state.rmq_connection_info)
}

#[debug_handler]
async fn list_queues(State(state): State<AppState>) -> Result<Json<Vec<QueueSummary>>, ApiError> {
    let guarded = state.guarded.lock().await;

    let remote_queues = guarded.rabbitmq.list_queues().await?;
    let local_queues = guarded.database.get_queues()?;

    let queues: Vec<_> = remote_queues
        .into_iter()
        .map(|remote_queue| {
            let local_queue = local_queues.iter().find(|q| q.name == remote_queue.name);
            QueueSummary {
                queue_id: local_queue.map(|x| x.id),
                name: remote_queue.name,
                message_count_in_rmq: remote_queue.message_count,
                message_count_in_db: local_queue.map(|x| x.message_count),
                exclusive: remote_queue.exclusive,
            }
        })
        .collect();

    Ok(Json::from(queues))
}

async fn get_messages(
    State(state): State<AppState>,
    Path(queue_id): Path<u64>,
) -> Result<Json<Vec<Message>>, ApiError> {
    let guarded = state.guarded.lock().await;
    let messages = guarded.database.get_messages(queue_id)?;
    Ok(Json(messages))
}

pub async fn send_messages() -> Result<()> {
    Ok(())
}

pub async fn delete_messages(
    State(state): State<AppState>,
    Json(request): Json<DeleteMessagesRequest>,
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

pub async fn load_messages_by_queue_name(
    State(state): State<AppState>,
    Query(query): Query<LoadMessagesByQueueNameQuery>,
) -> Result<Json<LoadMessagesByQueueNameResponse>, ApiError> {
    let guarded = state.guarded.lock().await;

    let queue_id = match guarded.database.find_queue_by_name(&query.queue_name)? {
        None => guarded.database.create_queue(&query.queue_name)?,
        Some(queue_id) => queue_id,
    };

    let rmq_messages = guarded
        .rabbitmq
        .load_messages(&query.queue_name)
        .await?
        .into_iter()
        .map(|x| x.payload)
        .collect::<Vec<_>>();

    if !rmq_messages.is_empty() {
        guarded.database.save_messages(queue_id, &rmq_messages)?;
    }

    let db_messages = guarded.database.get_messages(queue_id)?;

    Ok(Json(LoadMessagesByQueueNameResponse {
        messages: db_messages,
    }))
}
