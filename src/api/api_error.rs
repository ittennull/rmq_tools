use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use crate::database::DatabaseError;
use crate::rabbitmq::RabbitMQError;

pub enum ApiError{
    Database(DatabaseError),
    RabbitMQ(RabbitMQError),
    Http(anyhow::Error)
}

impl From<DatabaseError> for ApiError{
    fn from(value: DatabaseError) -> Self {
        ApiError::Database(value)
    }
}

impl From<RabbitMQError> for ApiError{
    fn from(value: RabbitMQError) -> Self {
        ApiError::RabbitMQ(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let text = match self {
            ApiError::Database(error) => {format!("{:?}", error.0)}
            ApiError::RabbitMQ(error) => {format!("{:?}", error.0)}
            ApiError::Http(error) => {format!("{:#}", error)}
        };
        (StatusCode::INTERNAL_SERVER_ERROR, text).into_response()
    }
}