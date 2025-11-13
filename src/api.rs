mod api_error;

use crate::api::api_error::ApiError;
use crate::database::{Database, MessageId, MessageSelector, QueueId};
use crate::dtos::{
    DeleteMessagesRequest, LoadMessagesByQueueNameQuery, LoadMessagesByQueueNameResponse, Message,
    PeekMessagesQuery, QueueSummary, RmqConnectionInfo, SendMessagesRequest,
};
use crate::rabbitmq::Rabbitmq;
use crate::rmq_background::RmqBackground;
use anyhow::Result;
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, Path, Query, State};
use axum::http::HeaderValue;
use axum::response::Response;
use axum::routing::{any, delete, get, post, put};
use axum::{Json, Router};
use log::{debug, error};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

struct GuardedData {
    rabbitmq: Rabbitmq,
    database: Database,
}

#[derive(Clone)]
struct AppState {
    guarded: Arc<Mutex<GuardedData>>,
    rmq_connection_info: RmqConnectionInfo,
    rmq_background: RmqBackground,
}

impl AppState {
    fn new(rabbitmq: Rabbitmq, database: Database, rmq_background: RmqBackground) -> Self {
        Self {
            rmq_connection_info: rabbitmq.get_connection_info(),
            guarded: Arc::new(Mutex::new(GuardedData { rabbitmq, database })),
            rmq_background,
        }
    }
}

pub fn build_api(
    rmq_client: Rabbitmq,
    database: Database,
    rmq_background: RmqBackground,
    wwwroot_dir: std::path::PathBuf,
) -> Router {
    let state = AppState::new(rmq_client, database, rmq_background);

    let mut index_html_path = wwwroot_dir.clone();
    index_html_path.push("index.html");

    // CORS for local development when UI runs on a different port
    let cors_layer = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin("http://localhost:5009".parse::<HeaderValue>().unwrap());

    Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/rmq_connection", get(get_rmq_connection_info))
                .route("/queue/load", post(load_messages_by_queue_name))
                .route("/queue/peek", get(peek_messages))
                .route("/queues", get(list_queues))
                .route("/queues/{queue_id}/messages", get(get_messages))
                .route("/queues/{queue_id}/messages", delete(delete_messages))
                .route("/queues/{queue_id}/messages/send", post(send_messages))
                .route(
                    "/queues/{queue_id}/messages/{message_id}",
                    put(update_message),
                )
                .route("/ws", any(ws_handler))
                .with_state(state)
                .layer(cors_layer),
        )
        .fallback_service(ServeDir::new(wwwroot_dir).fallback(ServeFile::new(index_html_path)))
}

async fn get_rmq_connection_info(State(state): State<AppState>) -> Json<RmqConnectionInfo> {
    Json(state.rmq_connection_info)
}

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
    Path(queue_id): Path<QueueId>,
) -> Result<Json<Vec<Message>>, ApiError> {
    let guarded = state.guarded.lock().await;
    let messages = guarded
        .database
        .get_messages(&MessageSelector::AllInQueue(queue_id))?;
    Ok(Json(messages))
}

async fn send_messages(
    State(state): State<AppState>,
    Path(queue_id): Path<QueueId>,
    Json(request): Json<SendMessagesRequest>,
) -> Result<(), ApiError> {
    let guarded = state.guarded.lock().await;

    // get messages from database
    let message_selector = match &request.message_ids[..] {
        &[] => MessageSelector::AllInQueue(queue_id),
        ids => MessageSelector::WithIds(ids),
    };
    let messages = guarded.database.get_messages(&message_selector)?;

    // publish messages
    for message in messages.into_iter() {
        guarded
            .rabbitmq
            .send_message(
                &request.destination_queue_name,
                &message.payload,
                message.headers,
            )
            .await?;
    }

    // delete messages
    guarded.database.delete_messages(&message_selector)?;

    Ok(())
}

async fn delete_messages(
    State(state): State<AppState>,
    Path(queue_id): Path<QueueId>,
    Json(request): Json<DeleteMessagesRequest>,
) -> Result<(), ApiError> {
    let message_selector = match &request.message_ids[..] {
        &[] => MessageSelector::AllInQueue(queue_id),
        ids => MessageSelector::WithIds(ids),
    };

    let guarded = state.guarded.lock().await;
    guarded.database.delete_messages(&message_selector)?;

    Ok(())
}

async fn load_messages_by_queue_name(
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
        .load_messages(&query.queue_name, false)
        .await?
        .into_iter()
        .map(|x| (x.payload, x.properties.0))
        .collect::<Vec<_>>();

    if !rmq_messages.is_empty() {
        guarded.database.save_messages(queue_id, &rmq_messages)?;
    }

    let messages = guarded
        .database
        .get_messages(&MessageSelector::AllInQueue(queue_id))?;

    Ok(Json(LoadMessagesByQueueNameResponse { queue_id, messages }))
}

async fn peek_messages(
    State(state): State<AppState>,
    Query(query): Query<PeekMessagesQuery>,
) -> Result<Json<Vec<Message>>, ApiError> {
    let guarded = state.guarded.lock().await;

    let rmq_messages = guarded
        .rabbitmq
        .load_messages(&query.queue_name, true)
        .await?
        .into_iter()
        .enumerate()
        .map(|(i, msg)| Message {
            id: i as MessageId,
            payload: msg.payload,
            headers: msg.properties.0,
        })
        .collect::<Vec<_>>();

    Ok(Json(rmq_messages))
}

async fn update_message(
    State(state): State<AppState>,
    Path((queue_id, message_id)): Path<(QueueId, MessageId)>,
    payload: String,
) -> Result<(), ApiError> {
    let guarded = state.guarded.lock().await;
    let changed = guarded
        .database
        .set_message_payload(queue_id, message_id, &payload)?;

    match changed {
        true => Ok(()),
        false => Err(ApiError::MessageNotFound(message_id)),
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    return ws.on_upgrade(move |socket| handle_socket(socket, state, addr));

    async fn handle_socket(mut socket: WebSocket, app_state: AppState, addr: SocketAddr) {
        debug!("Connected to websocket server from {}", addr);

        let mut receiver = app_state.rmq_background.subscribe();

        loop {
            if receiver.changed().await.is_err() {
                error!("tokio channel closed");
                break;
            }

            debug!("About to push data to websocket {}", addr);

            let json = {
                let counters = &*receiver.borrow_and_update();
                serde_json::to_string(counters).unwrap()
            };

            if socket
                .send(axum::extract::ws::Message::Text(json.into()))
                .await
                .is_err()
            {
                debug!("Client disconnected - {}", addr);
                return;
            }
        }
    }
}
