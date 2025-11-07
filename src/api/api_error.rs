use crate::database::{DatabaseError, MessageId};
use crate::rabbitmq::RabbitMQError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error: {}", .0)]
    Database(#[from] DatabaseError),

    #[error("RabbitMQ error: {}", .0)]
    RabbitMQ(#[from] RabbitMQError),

    #[error("Message not found: {}", .0)]
    MessageNotFound(MessageId),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = match self {
            ApiError::MessageNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status_code, self.to_string()).into_response()
    }
}
