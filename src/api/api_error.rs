use crate::database::DatabaseError;
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
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
